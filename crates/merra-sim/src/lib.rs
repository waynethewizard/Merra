//! Headless Bevy ECS orchestration for Merra.

use bevy_app::{App, Plugin};
use bevy_ecs::{
    entity::Entity,
    prelude::{Component, Query, Res, ResMut, Resource},
    schedule::{IntoScheduleConfigs, Schedule, ScheduleLabel, SystemSet},
};
use merra_core::{
    CalendarConfig, EVENT_SCHEMA_V1, EventId, EventKindV1, EventPayloadV1, PersonId,
    PersonRecordV1, PopulationConfigV1, RngDomain, ScenarioError, ScenarioV1, SimDuration, SimTime,
    SimulationSummaryV1, WorldEventV1, rng_for_domain,
};
use rand::RngExt;
use rand_chacha::ChaCha12Rng;
use thiserror::Error;

/// Orders the deterministic phases of a simulation step.
#[derive(Clone, Debug, Hash, Eq, PartialEq, SystemSet)]
pub enum SimulationSet {
    /// Advance the authoritative calendar.
    AdvanceTime,
    /// Age living people and evaluate mortality in stable identity order.
    Mortality,
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, ScheduleLabel)]
struct SimulationStep;

#[derive(Resource)]
struct Clock {
    now: SimTime,
}

#[derive(Resource)]
struct AdvanceRequest {
    duration: SimDuration,
}

#[derive(Resource)]
struct SimulationCalendar(CalendarConfig);

#[derive(Resource)]
struct PopulationRules(PopulationConfigV1);

#[derive(Resource)]
struct MortalityRng(ChaCha12Rng);

#[derive(Component)]
struct SimPerson {
    id: PersonId,
    name: String,
    starting_age_years: u16,
    age_days: u64,
    alive: bool,
    death_day: Option<u64>,
}

#[derive(Resource)]
struct EventLog {
    next_id: u64,
    events: Vec<WorldEventV1>,
}

impl EventLog {
    fn push(
        &mut self,
        time: SimTime,
        kind: EventKindV1,
        actors: Vec<PersonId>,
        causes: Vec<EventId>,
        tags: Vec<String>,
        payload: EventPayloadV1,
    ) -> EventId {
        let id = EventId(self.next_id);
        self.next_id += 1;
        self.events.push(WorldEventV1 {
            id,
            time,
            kind,
            actors,
            location: None,
            causes,
            tags,
            payload,
        });
        id
    }

    fn last_id(&self) -> Option<EventId> {
        self.events.last().map(|event| event.id)
    }
}

/// Installs deterministic simulation schedules without rendering or windowing.
pub struct MerraSimulationPlugin;

impl Plugin for MerraSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_schedule(Schedule::new(SimulationStep))
            .configure_sets(
                SimulationStep,
                (SimulationSet::AdvanceTime, SimulationSet::Mortality).chain(),
            )
            .add_systems(
                SimulationStep,
                advance_clock.in_set(SimulationSet::AdvanceTime),
            )
            .add_systems(
                SimulationStep,
                age_and_apply_mortality.in_set(SimulationSet::Mortality),
            );
    }
}

fn advance_clock(
    mut clock: ResMut<Clock>,
    request: Res<AdvanceRequest>,
    mut log: ResMut<EventLog>,
) {
    let from = clock.now;
    clock.now = clock.now.saturating_add(request.duration);
    let causes = log.last_id().into_iter().collect();
    log.push(
        clock.now,
        EventKindV1::TimeAdvanced,
        Vec::new(),
        causes,
        vec![String::from("time")],
        EventPayloadV1::TimeAdvanced {
            from_day: from.day(),
            to_day: clock.now.day(),
            elapsed_days: request.duration.days(),
        },
    );
}

fn age_and_apply_mortality(
    mut people: Query<(Entity, &mut SimPerson)>,
    clock: Res<Clock>,
    request: Res<AdvanceRequest>,
    calendar: Res<SimulationCalendar>,
    rules: Res<PopulationRules>,
    mut mortality_rng: ResMut<MortalityRng>,
    mut log: ResMut<EventLog>,
) {
    let mut ordered_people: Vec<(PersonId, Entity)> = people
        .iter()
        .filter_map(|(entity, person)| person.alive.then_some((person.id, entity)))
        .collect();
    ordered_people.sort_unstable_by_key(|(id, _)| *id);
    let causal_event = log.last_id();

    for (person_id, entity) in ordered_people {
        let Ok((_, mut person)) = people.get_mut(entity) else {
            continue;
        };
        person.age_days = person.age_days.saturating_add(request.duration.days());
        let age_years = person.age_days / u64::from(calendar.0.days_per_year);
        let annual_rate = rules.0.annual_mortality_per_10_000(age_years);
        let step_rate = u64::from(annual_rate)
            .saturating_mul(request.duration.days())
            .div_ceil(u64::from(calendar.0.days_per_year))
            .min(10_000) as u32;
        let roll = mortality_rng.0.random_range(0..10_000_u32);

        if roll < step_rate {
            person.alive = false;
            person.death_day = Some(clock.now.day());
            log.push(
                clock.now,
                EventKindV1::PersonDied,
                vec![person_id],
                causal_event.into_iter().collect(),
                vec![String::from("person"), String::from("death")],
                EventPayloadV1::PersonDied {
                    person_id,
                    name: person.name.clone(),
                    age_years,
                    annual_deaths_per_10_000: annual_rate,
                },
            );
        }
    }
}

/// Deterministic output independent of filesystem and source-control metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SimulationReport {
    /// Structured world events in stable order.
    pub events: Vec<WorldEventV1>,
    /// Compact machine-readable summary.
    pub summary: SimulationSummaryV1,
    /// Final inspectable state for every initialized person.
    pub people: Vec<PersonRecordV1>,
    /// Human-readable chronicle.
    pub chronicle: String,
}

/// A headless simulation facade that owns its Bevy world.
pub struct Simulation {
    app: App,
    scenario: ScenarioV1,
    seed: u64,
    finished: bool,
}

impl Simulation {
    /// Creates a simulation at its epoch and emits its first event.
    pub fn from_scenario(scenario: ScenarioV1, seed: u64) -> Result<Self, SimulationError> {
        scenario.validate()?;

        let mut event_log = EventLog {
            next_id: 1,
            events: Vec::new(),
        };
        event_log.push(
            SimTime::EPOCH,
            EventKindV1::SimulationStarted,
            Vec::new(),
            Vec::new(),
            vec![String::from("simulation")],
            EventPayloadV1::SimulationStarted {
                scenario_id: scenario.id.clone(),
                seed,
            },
        );
        if scenario.population.initial_people > 0 {
            let cause = event_log.last_id().into_iter().collect();
            event_log.push(
                SimTime::EPOCH,
                EventKindV1::PopulationInitialized,
                Vec::new(),
                cause,
                vec![String::from("population")],
                EventPayloadV1::PopulationInitialized {
                    people: scenario.population.initial_people,
                },
            );
        }

        let mut app = App::new();
        app.insert_resource(Clock {
            now: SimTime::EPOCH,
        })
        .insert_resource(AdvanceRequest {
            duration: SimDuration::default(),
        })
        .insert_resource(SimulationCalendar(scenario.calendar))
        .insert_resource(PopulationRules(scenario.population.clone()))
        .insert_resource(MortalityRng(rng_for_domain(seed, RngDomain::Mortality)))
        .insert_resource(event_log)
        .add_plugins(MerraSimulationPlugin);
        spawn_initial_population(&mut app, &scenario, seed);

        Ok(Self {
            app,
            scenario,
            seed,
            finished: false,
        })
    }

    /// Advances the simulation by one explicit deterministic duration.
    pub fn advance(&mut self, duration: SimDuration) -> Result<(), SimulationError> {
        if self.finished {
            return Err(SimulationError::AlreadyFinished);
        }
        let mut remaining_days = duration.days();
        let maximum_step = u64::from(self.scenario.calendar.days_per_year);
        while remaining_days > 0 {
            let step = SimDuration::from_days(remaining_days.min(maximum_step));
            self.app
                .world_mut()
                .resource_mut::<AdvanceRequest>()
                .duration = step;
            self.app.world_mut().run_schedule(SimulationStep);
            remaining_days -= step.days();
        }
        Ok(())
    }

    /// Marks the run complete and emits a final structured event.
    pub fn finish(&mut self) -> Result<(), SimulationError> {
        if self.finished {
            return Err(SimulationError::AlreadyFinished);
        }

        let now = self.app.world().resource::<Clock>().now;
        let elapsed_years = now.year(self.scenario.calendar);
        let mut log = self.app.world_mut().resource_mut::<EventLog>();
        let causes = log.last_id().into_iter().collect();
        log.push(
            now,
            EventKindV1::SimulationCompleted,
            Vec::new(),
            causes,
            vec![String::from("simulation")],
            EventPayloadV1::SimulationCompleted {
                final_day: now.day(),
                elapsed_years,
            },
        );
        self.finished = true;
        Ok(())
    }

    /// Creates deterministic machine and human reports.
    #[must_use]
    pub fn report(&self) -> SimulationReport {
        let events = self.app.world().resource::<EventLog>().events.clone();
        let now = self.app.world().resource::<Clock>().now;
        let elapsed_years = now.year(self.scenario.calendar);
        let mut people: Vec<PersonRecordV1> = self
            .app
            .world()
            .iter_entities()
            .filter_map(|entity| {
                let person = entity.get::<SimPerson>()?;
                Some(PersonRecordV1 {
                    id: person.id,
                    name: person.name.clone(),
                    starting_age_years: person.starting_age_years,
                    final_age_years: person.age_days
                        / u64::from(self.scenario.calendar.days_per_year),
                    alive: person.alive,
                    death_day: person.death_day,
                })
            })
            .collect();
        people.sort_unstable_by_key(|person| person.id);
        let living_population = people.iter().filter(|person| person.alive).count() as u32;
        let initial_population = people.len() as u32;
        let deaths = initial_population.saturating_sub(living_population);
        let summary = SimulationSummaryV1 {
            schema_version: EVENT_SCHEMA_V1,
            scenario_id: self.scenario.id.clone(),
            seed: self.seed,
            elapsed_days: now.day(),
            elapsed_years,
            days_per_year: self.scenario.calendar.days_per_year,
            event_count: events.len(),
            initial_population,
            living_population,
            deaths,
        };
        let chronicle = render_chronicle(
            &self.scenario,
            self.seed,
            &events,
            &people,
            now,
            elapsed_years,
        );

        SimulationReport {
            events,
            summary,
            people,
            chronicle,
        }
    }
}

/// Runs a whole number of scenario years and returns deterministic evidence.
pub fn run_years(
    scenario: ScenarioV1,
    seed: u64,
    years: u32,
) -> Result<SimulationReport, SimulationError> {
    let calendar = scenario.calendar;
    let mut simulation = Simulation::from_scenario(scenario, seed)?;
    simulation.advance(SimDuration::from_years(years, calendar))?;
    simulation.finish()?;
    Ok(simulation.report())
}

fn render_chronicle(
    scenario: &ScenarioV1,
    seed: u64,
    events: &[WorldEventV1],
    people: &[PersonRecordV1],
    now: SimTime,
    elapsed_years: u64,
) -> String {
    let year_unit = if elapsed_years == 1 { "year" } else { "years" };
    let living = people.iter().filter(|person| person.alive).count();
    let deaths = people.len().saturating_sub(living);
    let population_line = if people.is_empty() {
        String::from("- Population: no people initialized\n")
    } else {
        format!(
            "- Population: {} initialized, {living} living, {deaths} deaths\n",
            people.len()
        )
    };
    let notable_lives = render_notable_lives(people, scenario.calendar);
    format!(
        "# Chronicle: {}\n\n\
         - Scenario: `{}`\n\
         - Seed: `{seed}`\n\
         - Calendar: {} days per year\n\
         - Elapsed: {} days ({elapsed_years} complete {year_unit})\n\
         {population_line}\
         - Structured events: {}\n\n\
         The clock advanced deterministically from Day 0 to Day {}.\n\
         {notable_lives}",
        scenario.title,
        scenario.id,
        scenario.calendar.days_per_year,
        now.day(),
        events.len(),
        now.day(),
    )
}

fn render_notable_lives(people: &[PersonRecordV1], calendar: CalendarConfig) -> String {
    let first_death = people
        .iter()
        .filter_map(|person| person.death_day.map(|day| (day, person.id, person)))
        .min_by_key(|(day, id, _)| (*day, *id));
    let final_death = people
        .iter()
        .filter_map(|person| person.death_day.map(|day| (day, person.id, person)))
        .max_by_key(|(day, id, _)| (*day, *id));
    let longest_life = people
        .iter()
        .filter(|person| !person.alive)
        .max_by_key(|person| (person.final_age_years, std::cmp::Reverse(person.id)));

    let (Some((first_day, _, first)), Some((final_day, _, last)), Some(longest)) =
        (first_death, final_death, longest_life)
    else {
        return String::new();
    };

    format!(
        "\n## Notable Lives\n\n\
         - First recorded death: {} at age {} in Year {}.\n\
         - Longest recorded life: {} reached age {}.\n\
         - Final recorded death: {} at age {} in Year {}.\n",
        first.name,
        first.final_age_years,
        first_day / u64::from(calendar.days_per_year),
        longest.name,
        longest.final_age_years,
        last.name,
        last.final_age_years,
        final_day / u64::from(calendar.days_per_year),
    )
}

fn spawn_initial_population(app: &mut App, scenario: &ScenarioV1, seed: u64) {
    let config = &scenario.population;
    let mut population_rng = rng_for_domain(seed, RngDomain::Population);
    let mut name_rng = rng_for_domain(seed, RngDomain::Names);
    for raw_id in 1..=u64::from(config.initial_people) {
        let starting_age_years =
            population_rng.random_range(config.minimum_starting_age..=config.maximum_starting_age);
        let given = GIVEN_NAMES[name_rng.random_range(0..GIVEN_NAMES.len())];
        let family = FAMILY_NAMES[name_rng.random_range(0..FAMILY_NAMES.len())];
        app.world_mut().spawn(SimPerson {
            id: PersonId(raw_id),
            name: format!("{given} {family}"),
            starting_age_years,
            age_days: u64::from(starting_age_years)
                .saturating_mul(u64::from(scenario.calendar.days_per_year)),
            alive: true,
            death_day: None,
        });
    }
}

const GIVEN_NAMES: &[&str] = &[
    "Alda", "Ansel", "Bera", "Cerdic", "Dunstan", "Edith", "Elian", "Frida", "Garin", "Hilda",
    "Ivo", "Leof", "Mara", "Odel", "Runa", "Sella", "Toma", "Willa",
];

const FAMILY_NAMES: &[&str] = &[
    "Ash", "Barrow", "Bell", "Crow", "Dale", "Fen", "Gorse", "Hearth", "Marsh", "Mere", "Oak",
    "Reed", "Stone", "Thorn", "Vale", "Wold",
];

/// Simulation lifecycle or input failure.
#[derive(Debug, Error)]
pub enum SimulationError {
    /// Scenario validation failed.
    #[error(transparent)]
    Scenario(#[from] ScenarioError),
    /// A finished run cannot advance or finish twice.
    #[error("simulation has already finished")]
    AlreadyFinished,
}

#[cfg(test)]
mod tests {
    use merra_core::{
        CalendarConfig, EventPayloadV1, MortalityBandV1, PersonId, PopulationConfigV1,
        SCENARIO_SCHEMA_V1, ScenarioV1, SimDuration,
    };

    use super::Simulation;

    fn scenario() -> ScenarioV1 {
        ScenarioV1 {
            schema_version: SCENARIO_SCHEMA_V1,
            id: String::from("test"),
            title: String::from("Test"),
            calendar: CalendarConfig { days_per_year: 360 },
            population: PopulationConfigV1 {
                initial_people: 0,
                minimum_starting_age: 0,
                maximum_starting_age: 0,
                mortality_bands: Vec::new(),
            },
        }
    }

    #[test]
    fn same_inputs_produce_equal_reports() -> Result<(), Box<dyn std::error::Error>> {
        let mut first = Simulation::from_scenario(scenario(), 42)?;
        let mut second = Simulation::from_scenario(scenario(), 42)?;
        let duration = SimDuration::from_years(1, scenario().calendar);

        first.advance(duration)?;
        first.finish()?;
        second.advance(duration)?;
        second.finish()?;

        assert_eq!(first.report(), second.report());
        Ok(())
    }

    #[test]
    fn certain_mortality_emits_deaths_in_stable_person_order()
    -> Result<(), Box<dyn std::error::Error>> {
        let populated = ScenarioV1 {
            schema_version: SCENARIO_SCHEMA_V1,
            id: String::from("certain-mortality"),
            title: String::from("Certain Mortality"),
            calendar: CalendarConfig { days_per_year: 360 },
            population: PopulationConfigV1 {
                initial_people: 3,
                minimum_starting_age: 10,
                maximum_starting_age: 10,
                mortality_bands: vec![MortalityBandV1 {
                    through_age: u16::MAX,
                    annual_deaths_per_10_000: 10_000,
                }],
            },
        };
        let report = super::run_years(populated, 42, 1)?;
        let deaths: Vec<_> = report
            .events
            .iter()
            .filter_map(|event| match event.payload {
                EventPayloadV1::PersonDied { person_id, .. } => Some(person_id),
                _ => None,
            })
            .collect();

        assert_eq!(deaths, vec![PersonId(1), PersonId(2), PersonId(3)]);
        assert_eq!(report.summary.deaths, 3);
        assert!(
            report
                .people
                .iter()
                .all(|person| !person.alive && person.final_age_years == 11)
        );
        Ok(())
    }
}
