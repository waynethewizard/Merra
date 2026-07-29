//! Headless Bevy ECS orchestration for Merra.

mod history;
mod villages;

pub use history::{
    HistoricalReport, HistoricalSimulation, HistorySimulationError, run_history,
    run_history_on_graph,
};
pub use villages::{LocalHistoryError, regional_history, run_local_history};

use std::collections::{BTreeMap, BTreeSet};

use bevy_app::{App, Plugin};
use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, Component, Query, Res, ResMut, Resource},
    schedule::{IntoScheduleConfigs, Schedule, ScheduleLabel, SystemSet},
    system::SystemParam,
};
use merra_core::{
    CalendarConfig, EventId, EventKindV1, EventPayloadV1, FamilyConfigV1, HouseholdId,
    HouseholdRecordV1, ItemArchetypeV1, ItemConfigV1, ItemCustodyV1, ItemId, ItemRecordV1,
    ItemSourceRoleV1, ItemSourceV1, ItemStatusV1, OwnershipTransferReasonV1, PersonId,
    PersonRecordV1, PopulationConfigV1, PropertyOwnerV1, RngDomain, SUMMARY_SCHEMA_V1,
    ScenarioError, ScenarioV1, SimDuration, SimTime, SimulationSummaryV1, WorldEventV1,
    WorldSubjectV1, rng_for_domain,
};
use rand::RngExt;
use rand_chacha::ChaCha12Rng;
use thiserror::Error;

/// Orders the deterministic phases of a simulation step.
#[derive(Clone, Debug, Hash, Eq, PartialEq, SystemSet)]
pub enum SimulationSet {
    /// Advance the authoritative calendar.
    AdvanceTime,
    /// Record a named season when the clock reaches a boundary.
    SeasonTransition,
    /// Age living people and evaluate mortality in stable identity order.
    Mortality,
    /// Maintain partnerships and households, then create eligible births.
    Family,
    /// Maintain, use, transfer, and transform durable items.
    Items,
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
struct FamilyRules(FamilyConfigV1);

#[derive(Resource)]
struct ItemRules(ItemConfigV1);

#[derive(Resource)]
struct MortalityRng(ChaCha12Rng);

#[derive(Resource, Default)]
struct AnnualMortalityClock {
    elapsed_days: u64,
}

#[derive(Resource)]
struct FamilyRuntime {
    household_rng: ChaCha12Rng,
    name_rng: ChaCha12Rng,
    next_person_id: u64,
    next_household_id: u64,
}

#[derive(Resource)]
struct ItemRuntime {
    next_item_id: u64,
}

#[derive(SystemParam)]
struct MortalityInputs<'w> {
    clock: Res<'w, Clock>,
    request: Res<'w, AdvanceRequest>,
    calendar: Res<'w, SimulationCalendar>,
    rules: Res<'w, PopulationRules>,
}

#[derive(SystemParam)]
struct FamilyInputs<'w> {
    clock: Res<'w, Clock>,
    calendar: Res<'w, SimulationCalendar>,
    rules: Res<'w, FamilyRules>,
}

#[derive(Component)]
struct SimPerson {
    id: PersonId,
    name: String,
    given_name: String,
    surname: String,
    starting_age_years: u16,
    age_days: u64,
    alive: bool,
    death_day: Option<u64>,
    birth_day: Option<u64>,
    parent_ids: Vec<PersonId>,
    household_id: Option<HouseholdId>,
    partner_id: Option<PersonId>,
    generation: u16,
}

#[derive(Component)]
struct SimHousehold {
    id: HouseholdId,
    name: String,
    surname: String,
    member_ids: Vec<PersonId>,
    historical_member_ids: Vec<PersonId>,
    founded_day: u64,
    dissolved_day: Option<u64>,
    children_born: u16,
    last_birth_day: Option<u64>,
}

#[derive(Component)]
struct SimItem {
    id: ItemId,
    archetype_id: String,
    name: String,
    introduced_day: u64,
    introduction_event_id: EventId,
    sources: Vec<ItemSourceV1>,
    lineage_generation: u16,
    condition_per_10_000: u16,
    repairs: u16,
    status: ItemStatusV1,
    owner: PropertyOwnerV1,
    custody: ItemCustodyV1,
    last_event_id: EventId,
}

#[derive(Clone)]
struct PersonSnapshot {
    entity: Entity,
    id: PersonId,
    surname: String,
    age_years: u64,
    alive: bool,
    parent_ids: Vec<PersonId>,
    household_id: Option<HouseholdId>,
    partner_id: Option<PersonId>,
    generation: u16,
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
            subjects: item_subjects(&payload),
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

fn item_subjects(payload: &EventPayloadV1) -> Vec<WorldSubjectV1> {
    let item_ids: Vec<ItemId> = match payload {
        EventPayloadV1::ItemIntroduced { item_id, .. }
        | EventPayloadV1::ItemUsed { item_id, .. }
        | EventPayloadV1::ItemRepaired { item_id, .. }
        | EventPayloadV1::ItemOwnershipTransferred { item_id, .. }
        | EventPayloadV1::ItemCustodyTransferred { item_id, .. }
        | EventPayloadV1::ItemRelocated { item_id, .. }
        | EventPayloadV1::ItemLost { item_id, .. }
        | EventPayloadV1::ItemRecovered { item_id, .. }
        | EventPayloadV1::ItemDestroyed { item_id }
        | EventPayloadV1::HouseholdWorkCompleted { item_id, .. } => vec![*item_id],
        EventPayloadV1::ItemTransformed {
            source_item_ids,
            output_item_ids,
            ..
        } => source_item_ids
            .iter()
            .chain(output_item_ids)
            .copied()
            .collect(),
        _ => Vec::new(),
    };
    item_ids.into_iter().map(WorldSubjectV1::Item).collect()
}

/// Installs deterministic simulation schedules without rendering or windowing.
pub struct MerraSimulationPlugin;

impl Plugin for MerraSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_schedule(Schedule::new(SimulationStep))
            .configure_sets(
                SimulationStep,
                (
                    SimulationSet::AdvanceTime,
                    SimulationSet::SeasonTransition,
                    SimulationSet::Mortality,
                    SimulationSet::Family,
                    SimulationSet::Items,
                )
                    .chain(),
            )
            .add_systems(
                SimulationStep,
                advance_clock.in_set(SimulationSet::AdvanceTime),
            )
            .add_systems(
                SimulationStep,
                record_season_boundary.in_set(SimulationSet::SeasonTransition),
            )
            .add_systems(
                SimulationStep,
                age_and_apply_mortality.in_set(SimulationSet::Mortality),
            )
            .add_systems(
                SimulationStep,
                maintain_families.in_set(SimulationSet::Family),
            )
            .add_systems(SimulationStep, maintain_items.in_set(SimulationSet::Items));
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

fn record_season_boundary(
    clock: Res<Clock>,
    calendar: Res<SimulationCalendar>,
    mut log: ResMut<EventLog>,
) {
    let Some((year, season)) = calendar.0.season_starting_at_day(clock.now.day()) else {
        return;
    };
    let causes = log.last_id().into_iter().collect();
    log.push(
        clock.now,
        EventKindV1::SeasonBegan,
        Vec::new(),
        causes,
        vec![String::from("time"), String::from("season")],
        EventPayloadV1::SeasonBegan {
            season_id: season.id.clone(),
            season_name: season.name.clone(),
            year,
        },
    );
}

fn age_and_apply_mortality(
    mut people: Query<(Entity, &mut SimPerson)>,
    inputs: MortalityInputs<'_>,
    mut mortality_rng: ResMut<MortalityRng>,
    mut mortality_clock: ResMut<AnnualMortalityClock>,
    mut log: ResMut<EventLog>,
) {
    mortality_clock.elapsed_days = mortality_clock
        .elapsed_days
        .saturating_add(inputs.request.duration.days());
    let evaluate_mortality = inputs
        .clock
        .now
        .day_of_year(inputs.calendar.0.days_per_year)
        == 0
        && inputs.clock.now != SimTime::EPOCH;
    let mortality_days = mortality_clock.elapsed_days;
    if evaluate_mortality {
        mortality_clock.elapsed_days = 0;
    }

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
        person.age_days = person
            .age_days
            .saturating_add(inputs.request.duration.days());
        if !evaluate_mortality {
            continue;
        }
        let age_years = person.age_days / u64::from(inputs.calendar.0.days_per_year);
        let annual_rate = inputs.rules.0.annual_mortality_per_10_000(age_years);
        let step_rate = u64::from(annual_rate)
            .saturating_mul(mortality_days)
            .div_ceil(u64::from(inputs.calendar.0.days_per_year))
            .min(10_000) as u32;
        let roll = mortality_rng.0.random_range(0..10_000_u32);

        if roll < step_rate {
            person.alive = false;
            person.death_day = Some(inputs.clock.now.day());
            log.push(
                inputs.clock.now,
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

fn maintain_families(
    mut commands: Commands,
    mut people: Query<(Entity, &mut SimPerson)>,
    mut households: Query<&mut SimHousehold>,
    inputs: FamilyInputs<'_>,
    mut runtime: ResMut<FamilyRuntime>,
    mut log: ResMut<EventLog>,
) {
    if !inputs.rules.0.enabled
        || inputs
            .clock
            .now
            .day_of_year(inputs.calendar.0.days_per_year)
            != 0
        || inputs.clock.now == SimTime::EPOCH
    {
        return;
    }

    let now = inputs.clock.now;
    let days_per_year = u64::from(inputs.calendar.0.days_per_year);
    let mut snapshot = snapshot_people(&people, days_per_year);
    let entity_by_id: BTreeMap<_, _> = snapshot
        .iter()
        .map(|person| (person.id, person.entity))
        .collect();

    let mut ended_partnerships = Vec::new();
    for person in &snapshot {
        let Some(partner_id) = person.partner_id else {
            continue;
        };
        if person.id >= partner_id {
            continue;
        }
        let Some(partner) = snapshot.iter().find(|candidate| candidate.id == partner_id) else {
            continue;
        };
        let deceased_id = match (person.alive, partner.alive) {
            (false, _) => person.id,
            (_, false) => partner.id,
            (true, true) => continue,
        };
        ended_partnerships.push(([person.id, partner.id], deceased_id));
    }
    ended_partnerships.sort_unstable_by_key(|(partners, _)| *partners);

    for (partners, deceased_id) in ended_partnerships {
        for partner_id in partners {
            if let Some(entity) = entity_by_id.get(&partner_id)
                && let Ok((_, mut person)) = people.get_mut(*entity)
            {
                person.partner_id = None;
            }
        }
        let causes = log
            .events
            .iter()
            .rev()
            .find(|event| {
                event.kind == EventKindV1::PersonDied && event.actors.contains(&deceased_id)
            })
            .map(|event| event.id)
            .into_iter()
            .collect();
        log.push(
            now,
            EventKindV1::PartnershipEnded,
            partners.to_vec(),
            causes,
            vec![String::from("family"), String::from("partnership")],
            EventPayloadV1::PartnershipEnded {
                partners,
                deceased_id,
            },
        );
    }

    snapshot = snapshot_people(&people, days_per_year);
    let living_ids: BTreeSet<_> = snapshot
        .iter()
        .filter(|person| person.alive)
        .map(|person| person.id)
        .collect();
    for person in snapshot
        .iter()
        .filter(|person| !person.alive && person.household_id.is_some())
    {
        if let Ok((_, mut person)) = people.get_mut(person.entity) {
            person.household_id = None;
        }
    }
    let mut household_ids: Vec<_> = households
        .iter()
        .filter(|household| household.dissolved_day.is_none())
        .map(|household| household.id)
        .collect();
    household_ids.sort_unstable();
    for household_id in &household_ids {
        let Some(mut household) = households
            .iter_mut()
            .find(|household| household.id == *household_id)
        else {
            continue;
        };
        let previous_members = household.member_ids.clone();
        household
            .member_ids
            .retain(|person_id| living_ids.contains(person_id));
        if household.member_ids.is_empty() {
            household.dissolved_day = Some(now.day());
            let name = household.name.clone();
            let causes =
                log.events
                    .iter()
                    .rev()
                    .find(|event| {
                        event.kind == EventKindV1::PersonDied
                            && event
                                .actors
                                .iter()
                                .any(|actor| previous_members.contains(actor))
                    })
                    .or_else(|| {
                        log.events.iter().rev().find(|event| {
                            event.kind == EventKindV1::SeasonBegan && event.time == now
                        })
                    })
                    .map(|event| event.id)
                    .into_iter()
                    .collect();
            log.push(
                now,
                EventKindV1::HouseholdDissolved,
                Vec::new(),
                causes,
                vec![String::from("family"), String::from("household")],
                EventPayloadV1::HouseholdDissolved {
                    household_id: *household_id,
                    name,
                },
            );
        }
    }

    snapshot = snapshot_people(&people, days_per_year);
    let mut eligible: Vec<_> = snapshot
        .iter()
        .filter(|person| {
            person.alive
                && person.partner_id.is_none()
                && person.age_years >= u64::from(inputs.rules.0.minimum_partnership_age)
        })
        .cloned()
        .collect();
    eligible.sort_unstable_by_key(|person| (person.generation, person.id));
    let mut partnerships = Vec::new();
    while let Some(first) = eligible.first().cloned() {
        eligible.remove(0);
        let Some(index) = eligible.iter().position(|candidate| {
            candidate.generation == first.generation && !close_relatives(&first, candidate)
        }) else {
            continue;
        };
        let second = eligible.remove(index);
        partnerships.push((first, second));
    }

    let mut departure_causes = BTreeMap::new();
    for (first, second) in partnerships {
        let household_id = HouseholdId(runtime.next_household_id);
        runtime.next_household_id = runtime.next_household_id.saturating_add(1);
        let surname = if runtime.household_rng.random_bool(0.5) {
            first.surname.clone()
        } else {
            second.surname.clone()
        };
        let name = format!("{surname} household");
        let partners = [first.id, second.id];
        let mut previous_households = BTreeSet::new();

        for founder in [&first, &second] {
            if let Some(previous_id) = founder.household_id
                && let Some(mut previous) = households
                    .iter_mut()
                    .find(|household| household.id == previous_id)
            {
                previous_households.insert(previous_id);
                previous
                    .member_ids
                    .retain(|person_id| *person_id != founder.id);
            }
            if let Ok((_, mut person)) = people.get_mut(founder.entity) {
                person.household_id = Some(household_id);
                person.partner_id = Some(if founder.id == first.id {
                    second.id
                } else {
                    first.id
                });
            }
        }

        commands.spawn(SimHousehold {
            id: household_id,
            name: name.clone(),
            surname: surname.clone(),
            member_ids: partners.to_vec(),
            historical_member_ids: partners.to_vec(),
            founded_day: now.day(),
            dissolved_day: None,
            children_born: 0,
            last_birth_day: None,
        });
        let causes = log
            .events
            .iter()
            .rev()
            .find(|event| event.kind == EventKindV1::SeasonBegan && event.time == now)
            .map(|event| event.id)
            .into_iter()
            .collect();
        let household_event = log.push(
            now,
            EventKindV1::HouseholdFormed,
            partners.to_vec(),
            causes,
            vec![String::from("family"), String::from("household")],
            EventPayloadV1::HouseholdFormed {
                household_id,
                name,
                surname,
                member_ids: partners.to_vec(),
            },
        );
        let partnership_event = log.push(
            now,
            EventKindV1::PartnershipFormed,
            partners.to_vec(),
            vec![household_event],
            vec![String::from("family"), String::from("partnership")],
            EventPayloadV1::PartnershipFormed {
                household_id,
                partners,
            },
        );
        for previous_household in previous_households {
            departure_causes.insert(previous_household, partnership_event);
        }
    }

    for (household_id, cause) in departure_causes {
        let Some(mut household) = households
            .iter_mut()
            .find(|household| household.id == household_id)
        else {
            continue;
        };
        if household.dissolved_day.is_some() || !household.member_ids.is_empty() {
            continue;
        }
        household.dissolved_day = Some(now.day());
        let name = household.name.clone();
        log.push(
            now,
            EventKindV1::HouseholdDissolved,
            Vec::new(),
            vec![cause],
            vec![String::from("family"), String::from("household")],
            EventPayloadV1::HouseholdDissolved { household_id, name },
        );
    }

    snapshot = snapshot_people(&people, days_per_year);
    let people_by_id: BTreeMap<_, _> = snapshot
        .iter()
        .map(|person| (person.id, person.clone()))
        .collect();
    let interval_days =
        u64::from(inputs.rules.0.birth_interval_years).saturating_mul(days_per_year);
    let mut birth_households: Vec<_> = households
        .iter()
        .filter(|household| household.dissolved_day.is_none())
        .map(|household| household.id)
        .collect();
    birth_households.sort_unstable();

    for household_id in birth_households {
        let Some(mut household) = households
            .iter_mut()
            .find(|household| household.id == household_id)
        else {
            continue;
        };
        if household.children_born >= inputs.rules.0.maximum_children_per_household
            || household
                .last_birth_day
                .is_some_and(|day| now.day().saturating_sub(day) < interval_days)
        {
            continue;
        }
        let mut partners: Vec<_> = household
            .member_ids
            .iter()
            .filter_map(|person_id| people_by_id.get(person_id))
            .filter(|person| {
                person.alive
                    && person.partner_id.is_some()
                    && person.age_years >= u64::from(inputs.rules.0.minimum_parent_age)
                    && person.age_years <= u64::from(inputs.rules.0.maximum_parent_age)
            })
            .collect();
        partners.sort_unstable_by_key(|person| person.id);
        let Some(first) = partners.first() else {
            continue;
        };
        let Some(second) = partners
            .iter()
            .find(|candidate| first.partner_id == Some(candidate.id))
        else {
            continue;
        };
        let generation = first.generation.max(second.generation).saturating_add(1);
        if generation > inputs.rules.0.maximum_generation {
            continue;
        }

        let person_id = PersonId(runtime.next_person_id);
        runtime.next_person_id = runtime.next_person_id.saturating_add(1);
        let given_name =
            String::from(GIVEN_NAMES[runtime.name_rng.random_range(0..GIVEN_NAMES.len())]);
        let surname = household.surname.clone();
        let name = format!("{given_name} {surname}");
        let parent_ids = [first.id.min(second.id), first.id.max(second.id)];
        household.member_ids.push(person_id);
        household.historical_member_ids.push(person_id);
        household.member_ids.sort_unstable();
        household.children_born = household.children_born.saturating_add(1);
        household.last_birth_day = Some(now.day());
        commands.spawn(SimPerson {
            id: person_id,
            name: name.clone(),
            given_name,
            surname,
            starting_age_years: 0,
            age_days: 0,
            alive: true,
            death_day: None,
            birth_day: Some(now.day()),
            parent_ids: parent_ids.to_vec(),
            household_id: Some(household_id),
            partner_id: None,
            generation,
        });
        let mut causes: Vec<_> = log
            .events
            .iter()
            .filter(|event| {
                (event.kind == EventKindV1::SeasonBegan && event.time == now)
                    || matches!(
                        &event.payload,
                        EventPayloadV1::PartnershipFormed {
                            household_id: formed_household,
                            ..
                        } if *formed_household == household_id
                    )
            })
            .map(|event| event.id)
            .collect();
        causes.sort_unstable();
        causes.dedup();
        let mut actors = parent_ids.to_vec();
        actors.push(person_id);
        log.push(
            now,
            EventKindV1::PersonBorn,
            actors,
            causes,
            vec![
                String::from("person"),
                String::from("birth"),
                String::from("family"),
            ],
            EventPayloadV1::PersonBorn {
                person_id,
                name,
                parent_ids,
                household_id,
                generation,
            },
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn maintain_items(
    mut commands: Commands,
    clock: Res<Clock>,
    calendar: Res<SimulationCalendar>,
    rules: Res<ItemRules>,
    mut runtime: ResMut<ItemRuntime>,
    people: Query<&SimPerson>,
    households: Query<&SimHousehold>,
    mut items: Query<&mut SimItem>,
    mut log: ResMut<EventLog>,
) {
    if !rules.0.enabled
        || clock.now == SimTime::EPOCH
        || clock.now.day_of_year(calendar.0.days_per_year) != 0
    {
        return;
    }
    let now = clock.now;
    let archetypes: BTreeMap<_, _> = rules
        .0
        .archetypes
        .iter()
        .map(|archetype| (archetype.id.as_str(), archetype))
        .collect();
    let mut ordered: Vec<_> = items.iter().map(|item| item.id).collect();
    ordered.sort_unstable();

    for item_id in ordered {
        let Some(mut item) = items.iter_mut().find(|item| item.id == item_id) else {
            continue;
        };
        if item.status != ItemStatusV1::Active {
            continue;
        }

        if rules.0.household_formation_contributions
            && let PropertyOwnerV1::Household(owner_id) = item.owner.clone()
            && let Some(owner_household) =
                households.iter().find(|household| household.id == owner_id)
        {
            let mut destinations = log
                .events
                .iter()
                .filter_map(|event| {
                    let EventPayloadV1::HouseholdFormed {
                        household_id,
                        member_ids,
                        ..
                    } = &event.payload
                    else {
                        return None;
                    };
                    (event.time == now
                        && *household_id != owner_id
                        && member_ids.iter().any(|person_id| {
                            owner_household.historical_member_ids.contains(person_id)
                                && !owner_household.member_ids.contains(person_id)
                        }))
                    .then_some((*household_id, event.id))
                })
                .collect::<Vec<_>>();
            destinations.sort_unstable();
            if let Some((destination_id, formation_event)) = destinations.first().copied() {
                let from = item.owner.clone();
                let to = PropertyOwnerV1::Household(destination_id);
                let ownership_event = log.push(
                    now,
                    EventKindV1::ItemOwnershipTransferred,
                    Vec::new(),
                    vec![formation_event, item.last_event_id],
                    vec![
                        String::from("item"),
                        String::from("household"),
                        String::from("contribution"),
                    ],
                    EventPayloadV1::ItemOwnershipTransferred {
                        item_id,
                        from,
                        to: to.clone(),
                        reason: OwnershipTransferReasonV1::HouseholdFormation,
                    },
                );
                let previous_custody = item.custody.clone();
                let new_custody = ItemCustodyV1::Household(destination_id);
                let custody_event = log.push(
                    now,
                    EventKindV1::ItemCustodyTransferred,
                    Vec::new(),
                    vec![ownership_event],
                    vec![String::from("item"), String::from("custody")],
                    EventPayloadV1::ItemCustodyTransferred {
                        item_id,
                        from: previous_custody,
                        to: new_custody.clone(),
                    },
                );
                item.owner = to;
                item.custody = new_custody;
                item.last_event_id = custody_event;
            }
        }

        if let PropertyOwnerV1::Household(owner_id) = item.owner.clone()
            && households
                .iter()
                .find(|household| household.id == owner_id)
                .is_some_and(|household| household.dissolved_day.is_some())
            && let Some(heir_id) = nearest_living_heir_household(owner_id, &households, &people)
        {
            let from = item.owner.clone();
            let to = PropertyOwnerV1::Household(heir_id);
            let cause = log
                .events
                .iter()
                .rev()
                .find(|event| {
                    matches!(
                        event.payload,
                        EventPayloadV1::HouseholdDissolved { household_id, .. }
                            if household_id == owner_id
                    )
                })
                .map(|event| event.id)
                .into_iter()
                .collect();
            let ownership_event = log.push(
                now,
                EventKindV1::ItemOwnershipTransferred,
                Vec::new(),
                cause,
                vec![String::from("item"), String::from("inheritance")],
                EventPayloadV1::ItemOwnershipTransferred {
                    item_id,
                    from,
                    to: to.clone(),
                    reason: OwnershipTransferReasonV1::Inheritance,
                },
            );
            let previous_custody = item.custody.clone();
            let new_custody = ItemCustodyV1::Household(heir_id);
            let custody_event = log.push(
                now,
                EventKindV1::ItemCustodyTransferred,
                Vec::new(),
                vec![ownership_event],
                vec![String::from("item"), String::from("custody")],
                EventPayloadV1::ItemCustodyTransferred {
                    item_id,
                    from: previous_custody,
                    to: new_custody.clone(),
                },
            );
            item.owner = to;
            item.custody = new_custody;
            item.last_event_id = custody_event;
        }

        let Some(archetype) = archetypes.get(item.archetype_id.as_str()).copied() else {
            continue;
        };
        if item.condition_per_10_000 <= archetype.repair_below {
            if item.repairs < archetype.maximum_repairs {
                let before = item.condition_per_10_000;
                item.condition_per_10_000 = item
                    .condition_per_10_000
                    .saturating_add(archetype.repair_amount)
                    .min(10_000);
                item.repairs = item.repairs.saturating_add(1);
                item.last_event_id = log.push(
                    now,
                    EventKindV1::ItemRepaired,
                    Vec::new(),
                    vec![item.last_event_id],
                    vec![String::from("item"), String::from("maintenance")],
                    EventPayloadV1::ItemRepaired {
                        item_id,
                        condition_before_per_10_000: before,
                        condition_after_per_10_000: item.condition_per_10_000,
                        repair_number: item.repairs,
                    },
                );
            } else if let Some(target_id) = &archetype.rework_into
                && archetypes.contains_key(target_id.as_str())
            {
                let output_id = ItemId(runtime.next_item_id);
                runtime.next_item_id = runtime.next_item_id.saturating_add(1);
                let sources = vec![ItemSourceV1 {
                    item_id,
                    role: ItemSourceRoleV1::Material,
                }];
                let event_id = log.push(
                    now,
                    EventKindV1::ItemTransformed,
                    Vec::new(),
                    vec![item.last_event_id],
                    vec![String::from("item"), String::from("provenance")],
                    EventPayloadV1::ItemTransformed {
                        source_item_ids: vec![item_id],
                        output_item_ids: vec![output_id],
                        output_sources: vec![sources.clone()],
                    },
                );
                item.status = ItemStatusV1::Transformed;
                item.last_event_id = event_id;
                let root_name = item
                    .name
                    .split(" · reworked G")
                    .next()
                    .unwrap_or(item.name.as_str());
                let lineage_generation = item.lineage_generation.saturating_add(1);
                commands.spawn(SimItem {
                    id: output_id,
                    archetype_id: target_id.clone(),
                    name: format!("{root_name} · reworked G{lineage_generation}"),
                    introduced_day: now.day(),
                    introduction_event_id: event_id,
                    sources,
                    lineage_generation,
                    condition_per_10_000: 10_000,
                    repairs: 0,
                    status: ItemStatusV1::Active,
                    owner: item.owner.clone(),
                    custody: item.custody.clone(),
                    last_event_id: event_id,
                });
                continue;
            }
        }

        let before = item.condition_per_10_000;
        let productivity =
            u32::from(archetype.productivity_per_10_000).saturating_mul(u32::from(before)) / 10_000;
        item.condition_per_10_000 = before.saturating_sub(archetype.wear_per_use);
        item.last_event_id = log.push(
            now,
            EventKindV1::ItemUsed,
            Vec::new(),
            vec![item.last_event_id],
            vec![
                String::from("item"),
                String::from("work"),
                archetype.work_tag.clone(),
            ],
            EventPayloadV1::ItemUsed {
                item_id,
                work_tag: archetype.work_tag.clone(),
                productivity_per_10_000: productivity as u16,
                condition_before_per_10_000: before,
                condition_after_per_10_000: item.condition_per_10_000,
            },
        );
        if let ItemCustodyV1::Household(household_id) = &item.custody {
            log.push(
                now,
                EventKindV1::HouseholdWorkCompleted,
                Vec::new(),
                vec![item.last_event_id],
                vec![
                    String::from("household"),
                    String::from("work"),
                    archetype.work_tag.clone(),
                ],
                EventPayloadV1::HouseholdWorkCompleted {
                    household_id: *household_id,
                    item_id,
                    work_tag: archetype.work_tag.clone(),
                    base_labor: 10_000,
                    effective_labor: 10_000_u32.saturating_add(productivity),
                },
            );
        }
    }
}

fn nearest_living_heir_household(
    owner_id: HouseholdId,
    households: &Query<&SimHousehold>,
    people: &Query<&SimPerson>,
) -> Option<HouseholdId> {
    let owner = households
        .iter()
        .find(|household| household.id == owner_id)?;
    let ancestors: BTreeSet<_> = owner.historical_member_ids.iter().copied().collect();
    let mut distance = BTreeMap::<PersonId, u16>::new();
    for ancestor in &ancestors {
        distance.insert(*ancestor, 0);
    }
    for generation in 1..=32_u16 {
        let parents: BTreeSet<_> = distance
            .iter()
            .filter(|(_, value)| **value < generation)
            .map(|(id, _)| *id)
            .collect();
        let mut added = false;
        for person in people.iter() {
            if distance.contains_key(&person.id)
                || !person
                    .parent_ids
                    .iter()
                    .any(|parent| parents.contains(parent))
            {
                continue;
            }
            distance.insert(person.id, generation);
            added = true;
        }
        if !added {
            break;
        }
    }
    let mut candidates: Vec<_> = people
        .iter()
        .filter(|person| person.alive && person.household_id != Some(owner_id))
        .filter_map(|person| {
            Some((
                *distance.get(&person.id)?,
                person.birth_day.unwrap_or(0),
                person.id,
                person.household_id?,
            ))
        })
        .collect();
    candidates.sort_unstable();
    candidates.first().map(|candidate| candidate.3)
}

fn snapshot_people(
    people: &Query<(Entity, &mut SimPerson)>,
    days_per_year: u64,
) -> Vec<PersonSnapshot> {
    let mut snapshot: Vec<_> = people
        .iter()
        .map(|(entity, person)| PersonSnapshot {
            entity,
            id: person.id,
            surname: person.surname.clone(),
            age_years: person.age_days / days_per_year,
            alive: person.alive,
            parent_ids: person.parent_ids.clone(),
            household_id: person.household_id,
            partner_id: person.partner_id,
            generation: person.generation,
        })
        .collect();
    snapshot.sort_unstable_by_key(|person| person.id);
    snapshot
}

fn close_relatives(first: &PersonSnapshot, second: &PersonSnapshot) -> bool {
    first.parent_ids.contains(&second.id)
        || second.parent_ids.contains(&first.id)
        || first
            .parent_ids
            .iter()
            .any(|parent| second.parent_ids.contains(parent))
}

/// Deterministic output independent of filesystem and source-control metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SimulationReport {
    /// Human-readable scenario title used by inspectors and other presentation.
    pub scenario_title: String,
    /// Structured world events in stable order.
    pub events: Vec<WorldEventV1>,
    /// Compact machine-readable summary.
    pub summary: SimulationSummaryV1,
    /// Final inspectable state for founders and people born during the run.
    pub people: Vec<PersonRecordV1>,
    /// Final inspectable state for every household formed during the run.
    pub households: Vec<HouseholdRecordV1>,
    /// Final inspectable state and immutable provenance for durable items.
    pub items: Vec<ItemRecordV1>,
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
        let (_, initial_season) = scenario
            .calendar
            .season_starting_at_day(SimTime::EPOCH.day())
            .ok_or(SimulationError::MissingInitialSeason)?;
        let cause = event_log.last_id().into_iter().collect();
        event_log.push(
            SimTime::EPOCH,
            EventKindV1::SeasonBegan,
            Vec::new(),
            cause,
            vec![String::from("time"), String::from("season")],
            EventPayloadV1::SeasonBegan {
                season_id: initial_season.id.clone(),
                season_name: initial_season.name.clone(),
                year: 0,
            },
        );

        let mut app = App::new();
        app.insert_resource(Clock {
            now: SimTime::EPOCH,
        })
        .insert_resource(AdvanceRequest {
            duration: SimDuration::default(),
        })
        .insert_resource(SimulationCalendar(scenario.calendar.clone()))
        .insert_resource(PopulationRules(scenario.population.clone()))
        .insert_resource(FamilyRules(scenario.family.clone()))
        .insert_resource(ItemRules(scenario.items.clone()))
        .insert_resource(MortalityRng(rng_for_domain(seed, RngDomain::Mortality)))
        .insert_resource(AnnualMortalityClock::default())
        .insert_resource(event_log)
        .add_plugins(MerraSimulationPlugin);
        let mut name_rng = rng_for_domain(seed, RngDomain::Names);
        spawn_initial_population(&mut app, &scenario, seed, &mut name_rng);
        let mut household_rng = rng_for_domain(seed, RngDomain::Households);
        let next_household_id = initialize_families(&mut app, &scenario, &mut household_rng);
        let next_item_id = initialize_items(&mut app, &scenario);
        app.insert_resource(FamilyRuntime {
            household_rng,
            name_rng,
            next_person_id: u64::from(scenario.population.initial_people).saturating_add(1),
            next_household_id,
        })
        .insert_resource(ItemRuntime { next_item_id });

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
        while remaining_days > 0 {
            let now = self.app.world().resource::<Clock>().now;
            let boundary_days = self
                .scenario
                .calendar
                .days_until_next_season(now.day())
                .ok_or(SimulationError::MissingSeasonBoundary)?;
            let step = SimDuration::from_days(remaining_days.min(boundary_days));
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
        let elapsed_years = now.year(self.scenario.calendar.days_per_year);
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
        let elapsed_years = now.year(self.scenario.calendar.days_per_year);
        let mut people: Vec<PersonRecordV1> = self
            .app
            .world()
            .iter_entities()
            .filter_map(|entity| {
                let person = entity.get::<SimPerson>()?;
                Some(PersonRecordV1 {
                    id: person.id,
                    name: person.name.clone(),
                    given_name: person.given_name.clone(),
                    surname: person.surname.clone(),
                    starting_age_years: person.starting_age_years,
                    final_age_years: person.age_days
                        / u64::from(self.scenario.calendar.days_per_year),
                    alive: person.alive,
                    death_day: person.death_day,
                    birth_day: person.birth_day,
                    parent_ids: person.parent_ids.clone(),
                    household_id: person.household_id,
                    partner_id: person.partner_id,
                    generation: person.generation,
                })
            })
            .collect();
        people.sort_unstable_by_key(|person| person.id);
        let mut households: Vec<HouseholdRecordV1> = self
            .app
            .world()
            .iter_entities()
            .filter_map(|entity| {
                let household = entity.get::<SimHousehold>()?;
                let mut member_ids = household.member_ids.clone();
                member_ids.sort_unstable();
                Some(HouseholdRecordV1 {
                    id: household.id,
                    name: household.name.clone(),
                    surname: household.surname.clone(),
                    member_ids,
                    founded_day: household.founded_day,
                    dissolved_day: household.dissolved_day,
                    children_born: household.children_born,
                    residence_id: None,
                })
            })
            .collect();
        households.sort_unstable_by_key(|household| household.id);
        let mut items: Vec<ItemRecordV1> = self
            .app
            .world()
            .iter_entities()
            .filter_map(|entity| {
                let item = entity.get::<SimItem>()?;
                Some(ItemRecordV1 {
                    id: item.id,
                    archetype_id: item.archetype_id.clone(),
                    name: item.name.clone(),
                    introduced_day: item.introduced_day,
                    introduction_event_id: item.introduction_event_id,
                    sources: item.sources.clone(),
                    lineage_generation: item.lineage_generation,
                    condition_per_10_000: item.condition_per_10_000,
                    repairs: item.repairs,
                    status: item.status,
                    owner: item.owner.clone(),
                    custody: item.custody.clone(),
                    current_location_id: None,
                })
            })
            .collect();
        items.sort_unstable_by_key(|item| item.id);
        let living_population = people.iter().filter(|person| person.alive).count() as u32;
        let initial_population = self.scenario.population.initial_people;
        let deaths = people.iter().filter(|person| !person.alive).count() as u32;
        let summary = SimulationSummaryV1 {
            schema_version: SUMMARY_SCHEMA_V1,
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
            &households,
            now,
            elapsed_years,
        );

        SimulationReport {
            scenario_title: self.scenario.title.clone(),
            events,
            summary,
            people,
            households,
            items,
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
    let days_per_year = scenario.calendar.days_per_year;
    let mut simulation = Simulation::from_scenario(scenario, seed)?;
    simulation.advance(SimDuration::from_years(years, days_per_year))?;
    simulation.finish()?;
    Ok(simulation.report())
}

fn render_chronicle(
    scenario: &ScenarioV1,
    seed: u64,
    events: &[WorldEventV1],
    people: &[PersonRecordV1],
    households: &[HouseholdRecordV1],
    now: SimTime,
    elapsed_years: u64,
) -> String {
    let year_unit = if elapsed_years == 1 { "year" } else { "years" };
    let living = people.iter().filter(|person| person.alive).count();
    let deaths = people.len().saturating_sub(living);
    let births = people
        .iter()
        .filter(|person| person.birth_day.is_some())
        .count();
    let population_line = if people.is_empty() {
        String::from("- Population: no people initialized\n")
    } else if scenario.family.enabled {
        format!(
            "- Population: {} initialized, {births} born, {living} living, {deaths} deaths\n",
            scenario.population.initial_people
        )
    } else {
        format!(
            "- Population: {} initialized, {living} living, {deaths} deaths\n",
            people.len()
        )
    };
    let notable_lives = render_notable_lives(people, scenario.calendar.days_per_year);
    let family_line = if scenario.family.enabled {
        let generations = people
            .iter()
            .map(|person| person.generation)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        format!(
            "- Families: {} households formed across {generations} generations\n",
            households.len()
        )
    } else {
        String::new()
    };
    let seasons = scenario
        .calendar
        .seasons
        .iter()
        .map(|season| format!("{} ({} days)", season.name, season.days))
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "# Chronicle: {}\n\n\
         - Scenario: `{}`\n\
         - Seed: `{seed}`\n\
         - Calendar: {} days per year\n\
         - Seasons: {seasons}\n\
         - Elapsed: {} days ({elapsed_years} complete {year_unit})\n\
         {population_line}\
         {family_line}\
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

fn render_notable_lives(people: &[PersonRecordV1], days_per_year: u16) -> String {
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
        first_day / u64::from(days_per_year),
        longest.name,
        longest.final_age_years,
        last.name,
        last.final_age_years,
        final_day / u64::from(days_per_year),
    )
}

fn spawn_initial_population(
    app: &mut App,
    scenario: &ScenarioV1,
    seed: u64,
    name_rng: &mut ChaCha12Rng,
) {
    let config = &scenario.population;
    let mut population_rng = rng_for_domain(seed, RngDomain::Population);
    for raw_id in 1..=u64::from(config.initial_people) {
        let starting_age_years =
            population_rng.random_range(config.minimum_starting_age..=config.maximum_starting_age);
        let given = String::from(GIVEN_NAMES[name_rng.random_range(0..GIVEN_NAMES.len())]);
        let family = String::from(FAMILY_NAMES[name_rng.random_range(0..FAMILY_NAMES.len())]);
        app.world_mut().spawn(SimPerson {
            id: PersonId(raw_id),
            name: format!("{given} {family}"),
            given_name: given,
            surname: family,
            starting_age_years,
            age_days: u64::from(starting_age_years)
                .saturating_mul(u64::from(scenario.calendar.days_per_year)),
            alive: true,
            death_day: None,
            birth_day: None,
            parent_ids: Vec::new(),
            household_id: None,
            partner_id: None,
            generation: 0,
        });
    }
}

fn initialize_families(
    app: &mut App,
    scenario: &ScenarioV1,
    household_rng: &mut ChaCha12Rng,
) -> u64 {
    if !scenario.family.enabled {
        return 1;
    }
    let mut founders: Vec<_> = app
        .world()
        .iter_entities()
        .filter_map(|entity| {
            let person = entity.get::<SimPerson>()?;
            Some((
                entity.id(),
                person.id,
                person.surname.clone(),
                person.age_days / u64::from(scenario.calendar.days_per_year),
            ))
        })
        .collect();
    founders.sort_unstable_by_key(|(_, person_id, _, _)| *person_id);

    let mut groups = Vec::new();
    let mut paired = BTreeSet::new();
    let eligible: Vec<_> = founders
        .iter()
        .filter(|(_, _, _, age)| *age >= u64::from(scenario.family.minimum_partnership_age))
        .cloned()
        .collect();
    for pair in eligible.chunks_exact(2) {
        paired.insert(pair[0].1);
        paired.insert(pair[1].1);
        groups.push(vec![pair[0].clone(), pair[1].clone()]);
    }
    groups.extend(
        founders
            .into_iter()
            .filter(|(_, person_id, _, _)| !paired.contains(person_id))
            .map(|founder| vec![founder]),
    );
    groups.sort_unstable_by_key(|group| group[0].1);

    let mut next_household_id = 1_u64;
    for group in groups {
        let household_id = HouseholdId(next_household_id);
        next_household_id = next_household_id.saturating_add(1);
        let surname = if group.len() == 2 && household_rng.random_bool(0.5) {
            group[1].2.clone()
        } else {
            group[0].2.clone()
        };
        let name = format!("{surname} household");
        let member_ids: Vec<_> = group.iter().map(|(_, id, _, _)| *id).collect();
        for (entity, person_id, _, _) in &group {
            if let Some(mut person) = app.world_mut().get_mut::<SimPerson>(*entity) {
                person.household_id = Some(household_id);
                if member_ids.len() == 2 {
                    person.partner_id = member_ids
                        .iter()
                        .copied()
                        .find(|candidate| *candidate != *person_id);
                }
            }
        }
        app.world_mut().spawn(SimHousehold {
            id: household_id,
            name: name.clone(),
            surname: surname.clone(),
            member_ids: member_ids.clone(),
            historical_member_ids: member_ids.clone(),
            founded_day: 0,
            dissolved_day: None,
            children_born: 0,
            last_birth_day: None,
        });
        let household_event = {
            let mut log = app.world_mut().resource_mut::<EventLog>();
            let causes = log
                .events
                .iter()
                .rev()
                .find(|event| event.kind == EventKindV1::SeasonBegan)
                .map(|event| event.id)
                .into_iter()
                .collect();
            log.push(
                SimTime::EPOCH,
                EventKindV1::HouseholdFormed,
                member_ids.clone(),
                causes,
                vec![String::from("family"), String::from("household")],
                EventPayloadV1::HouseholdFormed {
                    household_id,
                    name,
                    surname,
                    member_ids: member_ids.clone(),
                },
            )
        };
        if member_ids.len() == 2 {
            let partners = [member_ids[0], member_ids[1]];
            app.world_mut().resource_mut::<EventLog>().push(
                SimTime::EPOCH,
                EventKindV1::PartnershipFormed,
                member_ids,
                vec![household_event],
                vec![String::from("family"), String::from("partnership")],
                EventPayloadV1::PartnershipFormed {
                    household_id,
                    partners,
                },
            );
        }
    }
    next_household_id
}

fn initialize_items(app: &mut App, scenario: &ScenarioV1) -> u64 {
    if !scenario.items.enabled {
        return 1;
    }
    let mut households: Vec<_> = app
        .world()
        .iter_entities()
        .filter_map(|entity| {
            let household = entity.get::<SimHousehold>()?;
            Some((household.id, household.surname.clone()))
        })
        .collect();
    households.sort_unstable_by_key(|(id, _)| *id);
    let mut archetypes: Vec<&ItemArchetypeV1> = scenario
        .items
        .archetypes
        .iter()
        .filter(|archetype| archetype.initially_distributed)
        .collect();
    archetypes.sort_unstable_by_key(|archetype| archetype.id.as_str());
    let mut next_item_id = 1_u64;
    for (household_id, surname) in households {
        for archetype in &archetypes {
            for ordinal in 0..scenario.items.initial_items_per_household {
                let item_id = ItemId(next_item_id);
                next_item_id = next_item_id.saturating_add(1);
                let owner = PropertyOwnerV1::Household(household_id);
                let custody = ItemCustodyV1::Household(household_id);
                let name = if scenario.items.initial_items_per_household == 1 {
                    format!("{surname} {}", archetype.name)
                } else {
                    format!("{surname} {} {}", archetype.name, ordinal + 1)
                };
                let event_id = {
                    let mut log = app.world_mut().resource_mut::<EventLog>();
                    let causes = log
                        .events
                        .iter()
                        .rev()
                        .find(|event| {
                            matches!(
                                event.payload,
                                EventPayloadV1::HouseholdFormed {
                                    household_id: formed_id,
                                    ..
                                } if formed_id == household_id
                            )
                        })
                        .map(|event| event.id)
                        .into_iter()
                        .collect();
                    log.push(
                        SimTime::EPOCH,
                        EventKindV1::ItemIntroduced,
                        Vec::new(),
                        causes,
                        vec![
                            String::from("item"),
                            String::from("provenance"),
                            archetype.work_tag.clone(),
                        ],
                        EventPayloadV1::ItemIntroduced {
                            item_id,
                            archetype_id: archetype.id.clone(),
                            name: name.clone(),
                            owner: owner.clone(),
                            custody: custody.clone(),
                        },
                    )
                };
                app.world_mut().spawn(SimItem {
                    id: item_id,
                    archetype_id: archetype.id.clone(),
                    name,
                    introduced_day: 0,
                    introduction_event_id: event_id,
                    sources: Vec::new(),
                    lineage_generation: 0,
                    condition_per_10_000: 10_000,
                    repairs: 0,
                    status: ItemStatusV1::Active,
                    owner,
                    custody,
                    last_event_id: event_id,
                });
            }
        }
    }
    next_item_id
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
    /// A validated calendar unexpectedly had no season at the epoch.
    #[error("validated calendar has no initial season")]
    MissingInitialSeason,
    /// A validated calendar unexpectedly had no future season boundary.
    #[error("validated calendar has no next season boundary")]
    MissingSeasonBoundary,
    /// A finished run cannot advance or finish twice.
    #[error("simulation has already finished")]
    AlreadyFinished,
}

#[cfg(test)]
mod tests {
    use merra_core::{
        CalendarConfig, EventPayloadV1, FamilyConfigV1, MortalityBandV1, PersonId,
        PopulationConfigV1, SCENARIO_SCHEMA_V1, ScenarioV1, SeasonConfigV1, SimDuration,
    };

    use super::Simulation;

    fn calendar() -> CalendarConfig {
        CalendarConfig {
            days_per_year: 360,
            seasons: vec![
                SeasonConfigV1 {
                    id: String::from("thaw"),
                    name: String::from("Thaw"),
                    days: 90,
                },
                SeasonConfigV1 {
                    id: String::from("bloom"),
                    name: String::from("Bloom"),
                    days: 90,
                },
                SeasonConfigV1 {
                    id: String::from("highsun"),
                    name: String::from("Highsun"),
                    days: 90,
                },
                SeasonConfigV1 {
                    id: String::from("emberfall"),
                    name: String::from("Emberfall"),
                    days: 90,
                },
            ],
        }
    }

    fn scenario() -> ScenarioV1 {
        ScenarioV1 {
            schema_version: SCENARIO_SCHEMA_V1,
            id: String::from("test"),
            title: String::from("Test"),
            calendar: calendar(),
            population: PopulationConfigV1 {
                initial_people: 0,
                minimum_starting_age: 0,
                maximum_starting_age: 0,
                mortality_bands: Vec::new(),
            },
            family: FamilyConfigV1::default(),
            items: Default::default(),
        }
    }

    #[test]
    fn same_inputs_produce_equal_reports() -> Result<(), Box<dyn std::error::Error>> {
        let mut first = Simulation::from_scenario(scenario(), 42)?;
        let mut second = Simulation::from_scenario(scenario(), 42)?;
        let duration = SimDuration::from_years(1, scenario().calendar.days_per_year);

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
            calendar: calendar(),
            population: PopulationConfigV1 {
                initial_people: 3,
                minimum_starting_age: 10,
                maximum_starting_age: 10,
                mortality_bands: vec![MortalityBandV1 {
                    through_age: u16::MAX,
                    annual_deaths_per_10_000: 10_000,
                }],
            },
            family: FamilyConfigV1::default(),
            items: Default::default(),
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

    #[test]
    fn one_year_records_each_named_season_boundary() -> Result<(), Box<dyn std::error::Error>> {
        let report = super::run_years(scenario(), 42, 1)?;
        let seasons: Vec<_> = report
            .events
            .iter()
            .filter_map(|event| match &event.payload {
                EventPayloadV1::SeasonBegan {
                    season_id, year, ..
                } => Some((season_id.as_str(), *year)),
                _ => None,
            })
            .collect();

        assert_eq!(
            seasons,
            vec![
                ("thaw", 0),
                ("bloom", 0),
                ("highsun", 0),
                ("emberfall", 0),
                ("thaw", 1),
            ]
        );
        Ok(())
    }

    #[test]
    fn caller_step_size_does_not_change_annual_mortality() -> Result<(), Box<dyn std::error::Error>>
    {
        let populated = ScenarioV1 {
            schema_version: SCENARIO_SCHEMA_V1,
            id: String::from("step-size-invariance"),
            title: String::from("Step Size Invariance"),
            calendar: calendar(),
            population: PopulationConfigV1 {
                initial_people: 24,
                minimum_starting_age: 0,
                maximum_starting_age: 70,
                mortality_bands: vec![MortalityBandV1 {
                    through_age: u16::MAX,
                    annual_deaths_per_10_000: 5_000,
                }],
            },
            family: FamilyConfigV1::default(),
            items: Default::default(),
        };
        let mut whole_year = Simulation::from_scenario(populated.clone(), 42)?;
        let mut uneven_steps = Simulation::from_scenario(populated, 42)?;

        whole_year.advance(SimDuration::from_days(360))?;
        uneven_steps.advance(SimDuration::from_days(17))?;
        uneven_steps.advance(SimDuration::from_days(73))?;
        uneven_steps.advance(SimDuration::from_days(101))?;
        uneven_steps.advance(SimDuration::from_days(169))?;

        whole_year.finish()?;
        uneven_steps.finish()?;
        let whole_year = whole_year.report();
        let uneven_steps = uneven_steps.report();
        let death_payloads = |report: &super::SimulationReport| {
            report
                .events
                .iter()
                .filter_map(|event| match &event.payload {
                    EventPayloadV1::PersonDied { .. } => Some(event.payload.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
        };

        assert_eq!(whole_year.people, uneven_steps.people);
        assert_eq!(death_payloads(&whole_year), death_payloads(&uneven_steps));
        assert_eq!(
            whole_year.summary.living_population,
            uneven_steps.summary.living_population
        );
        Ok(())
    }

    #[test]
    fn deterministic_households_reach_four_generations() -> Result<(), Box<dyn std::error::Error>> {
        let families = ScenarioV1 {
            schema_version: SCENARIO_SCHEMA_V1,
            id: String::from("four-generations"),
            title: String::from("Four Generations"),
            calendar: calendar(),
            population: PopulationConfigV1 {
                initial_people: 8,
                minimum_starting_age: 18,
                maximum_starting_age: 18,
                mortality_bands: vec![MortalityBandV1 {
                    through_age: u16::MAX,
                    annual_deaths_per_10_000: 0,
                }],
            },
            family: FamilyConfigV1 {
                enabled: true,
                minimum_partnership_age: 18,
                minimum_parent_age: 20,
                maximum_parent_age: 40,
                birth_interval_years: 4,
                maximum_children_per_household: 2,
                maximum_generation: 3,
            },
            items: Default::default(),
        };
        let report = super::run_years(families, 42, 45)?;

        assert_eq!(
            report.people.iter().map(|person| person.generation).max(),
            Some(3)
        );
        assert!(report.households.len() >= 8);
        assert!(
            report
                .people
                .iter()
                .filter(|person| person.birth_day.is_some())
                .all(|person| {
                    person.parent_ids.len() == 2
                        && person
                            .parent_ids
                            .iter()
                            .all(|parent_id| *parent_id < person.id)
                })
        );

        for person in report.people.iter().filter(|person| person.alive) {
            let Some(partner_id) = person.partner_id else {
                continue;
            };
            let partner = report
                .people
                .iter()
                .find(|candidate| candidate.id == partner_id);
            assert!(partner.is_some_and(|partner| {
                partner.partner_id == Some(person.id)
                    && !person
                        .parent_ids
                        .iter()
                        .any(|parent| partner.parent_ids.contains(parent))
            }));
        }
        Ok(())
    }

    #[test]
    fn household_membership_is_current_after_death_and_departure()
    -> Result<(), Box<dyn std::error::Error>> {
        let family = FamilyConfigV1 {
            enabled: true,
            minimum_partnership_age: 18,
            minimum_parent_age: 100,
            maximum_parent_age: 100,
            birth_interval_years: 1,
            maximum_children_per_household: 1,
            maximum_generation: 1,
        };
        let mortality_bands = |rate| {
            vec![MortalityBandV1 {
                through_age: u16::MAX,
                annual_deaths_per_10_000: rate,
            }]
        };
        let moving_founders = ScenarioV1 {
            schema_version: SCENARIO_SCHEMA_V1,
            id: String::from("household-departure"),
            title: String::from("Household Departure"),
            calendar: calendar(),
            population: PopulationConfigV1 {
                initial_people: 3,
                minimum_starting_age: 17,
                maximum_starting_age: 17,
                mortality_bands: mortality_bands(0),
            },
            family: family.clone(),
            items: Default::default(),
        };
        let moved = super::run_years(moving_founders, 42, 1)?;

        assert!(
            moved
                .households
                .iter()
                .all(|household| household.dissolved_day.is_some()
                    || !household.member_ids.is_empty())
        );
        assert_eq!(
            moved
                .households
                .iter()
                .filter(|household| household.dissolved_day == Some(360))
                .count(),
            2
        );
        for person in &moved.people {
            let household = person.household_id.and_then(|household_id| {
                moved
                    .households
                    .iter()
                    .find(|household| household.id == household_id)
            });
            assert!(household.is_some_and(|household| household.member_ids.contains(&person.id)));
        }

        let dying_founders = ScenarioV1 {
            schema_version: SCENARIO_SCHEMA_V1,
            id: String::from("household-death"),
            title: String::from("Household Death"),
            calendar: calendar(),
            population: PopulationConfigV1 {
                initial_people: 2,
                minimum_starting_age: 18,
                maximum_starting_age: 18,
                mortality_bands: mortality_bands(10_000),
            },
            family,
            items: Default::default(),
        };
        let died = super::run_years(dying_founders, 42, 1)?;

        assert!(died.people.iter().all(|person| !person.alive
            && person.household_id.is_none()
            && person.partner_id.is_none()));
        assert!(died.households.iter().all(
            |household| household.dissolved_day == Some(360) && household.member_ids.is_empty()
        ));
        Ok(())
    }
}
