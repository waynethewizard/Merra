//! Headless Bevy ECS orchestration for Merra.

use bevy_app::{App, Plugin};
use bevy_ecs::{
    prelude::{Res, ResMut, Resource},
    schedule::{IntoScheduleConfigs, Schedule, ScheduleLabel, SystemSet},
};
use merra_core::{
    EVENT_SCHEMA_V1, EventId, EventKindV1, EventPayloadV1, ScenarioError, ScenarioV1, SimDuration,
    SimTime, SimulationSummaryV1, WorldEventV1,
};
use thiserror::Error;

/// Orders the deterministic phases of a simulation step.
#[derive(Clone, Debug, Hash, Eq, PartialEq, SystemSet)]
pub enum SimulationSet {
    /// Advance the authoritative calendar.
    AdvanceTime,
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
struct EventLog {
    next_id: u64,
    events: Vec<WorldEventV1>,
}

impl EventLog {
    fn push(
        &mut self,
        time: SimTime,
        kind: EventKindV1,
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
            actors: Vec::new(),
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
            .configure_sets(SimulationStep, SimulationSet::AdvanceTime)
            .add_systems(
                SimulationStep,
                advance_clock.in_set(SimulationSet::AdvanceTime),
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
        causes,
        vec![String::from("time")],
        EventPayloadV1::TimeAdvanced {
            from_day: from.day(),
            to_day: clock.now.day(),
            elapsed_days: request.duration.days(),
        },
    );
}

/// Deterministic output independent of filesystem and source-control metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SimulationReport {
    /// Structured world events in stable order.
    pub events: Vec<WorldEventV1>,
    /// Compact machine-readable summary.
    pub summary: SimulationSummaryV1,
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
            vec![String::from("simulation")],
            EventPayloadV1::SimulationStarted {
                scenario_id: scenario.id.clone(),
                seed,
            },
        );

        let mut app = App::new();
        app.insert_resource(Clock {
            now: SimTime::EPOCH,
        })
        .insert_resource(AdvanceRequest {
            duration: SimDuration::default(),
        })
        .insert_resource(event_log)
        .add_plugins(MerraSimulationPlugin);

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
        self.app
            .world_mut()
            .resource_mut::<AdvanceRequest>()
            .duration = duration;
        self.app.world_mut().run_schedule(SimulationStep);
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
        let summary = SimulationSummaryV1 {
            schema_version: EVENT_SCHEMA_V1,
            scenario_id: self.scenario.id.clone(),
            seed: self.seed,
            elapsed_days: now.day(),
            elapsed_years,
            event_count: events.len(),
        };
        let chronicle = render_chronicle(&self.scenario, self.seed, &events, now, elapsed_years);

        SimulationReport {
            events,
            summary,
            chronicle,
        }
    }
}

fn render_chronicle(
    scenario: &ScenarioV1,
    seed: u64,
    events: &[WorldEventV1],
    now: SimTime,
    elapsed_years: u64,
) -> String {
    let year_unit = if elapsed_years == 1 { "year" } else { "years" };
    format!(
        "# Chronicle: {}\n\n\
         - Scenario: `{}`\n\
         - Seed: `{seed}`\n\
         - Calendar: {} days per year\n\
         - Elapsed: {} days ({elapsed_years} complete {year_unit})\n\
         - Structured events: {}\n\n\
         The clock advanced deterministically from Day 0 to Day {}.\n",
        scenario.title,
        scenario.id,
        scenario.calendar.days_per_year,
        now.day(),
        events.len(),
        now.day(),
    )
}

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
    use merra_core::{CalendarConfig, SCENARIO_SCHEMA_V1, ScenarioV1, SimDuration};

    use super::Simulation;

    fn scenario() -> ScenarioV1 {
        ScenarioV1 {
            schema_version: SCENARIO_SCHEMA_V1,
            id: String::from("test"),
            title: String::from("Test"),
            calendar: CalendarConfig { days_per_year: 360 },
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
}
