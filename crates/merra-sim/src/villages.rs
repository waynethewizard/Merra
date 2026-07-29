//! Detailed household projection over the five-settlement historical handoff.

use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, BinaryHeap},
};

use merra_core::{
    EventId, EventKindV1, EventPayloadV1, HouseholdHistoricalContextV1, HouseholdId,
    LOCAL_HISTORY_SCHEMA_V1, LocalConnectionV1, LocalHistoryConfigError, LocalHistoryConfigV1,
    LocalHistoryReportV1, LocalHistorySummaryV1, LocalSettlementRecordV1, LocationId,
    PopulationAllocationV1, RegionalHistoryV1, ResidenceDecisionV1, ResidenceReasonV1, RngDomain,
    RouteId, RouteRecordV1, SimTime, SurfaceWorldV1, WorldEventV1, seed_for_domain,
};
use thiserror::Error;

use crate::{HistoricalReport, SimulationError, run_years};

/// Failure to project a completed aggregate history into detailed settlements.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum LocalHistoryError {
    /// Invalid local-history rules.
    #[error(transparent)]
    Config(#[from] LocalHistoryConfigError),
    /// Detailed person simulation failed.
    #[error("{0}")]
    Simulation(String),
    /// The selected handoff is not the configured five-settlement shape.
    #[error("starting region contains {found} settlements; expected {expected}")]
    SettlementCount { expected: usize, found: usize },
    /// A selected settlement does not resolve in the source place graph.
    #[error("selected location #{0} is missing from the source world")]
    MissingLocation(u64),
    /// A selected settlement has no aggregate population to reconcile.
    #[error("selected location #{0} has no aggregate population")]
    MissingPopulation(u64),
    /// The detailed sample did not create enough initial households.
    #[error("detailed sample created {found} initial households; at least {required} are required")]
    InsufficientInitialHouseholds { required: usize, found: usize },
    /// Two selected places have no available historical route.
    #[error("no open route connects selected locations #{from} and #{to}")]
    DisconnectedRegion { from: u64, to: u64 },
    /// The weighted route graph contains an invalid or duplicate route.
    #[error("route #{0} has invalid identity, endpoints, or travel cost")]
    InvalidRoute(u64),
    /// The historical handoff claims a route that is absent from the world.
    #[error("historical handoff opens unknown route #{0}")]
    UnknownOpenRoute(u64),
    /// An aggregate population could not be allocated exactly.
    #[error("macro-to-household projection did not reconcile exactly")]
    ProjectionMismatch,
    /// A local event references a household that was never assigned a residence.
    #[error("household #{0} has no local residence")]
    MissingHouseholdResidence(u64),
}

impl From<SimulationError> for LocalHistoryError {
    fn from(value: SimulationError) -> Self {
        Self::Simulation(value.to_string())
    }
}

/// Extracts the durable five-settlement handoff from a completed macro report.
#[must_use]
pub fn regional_history(report: &HistoricalReport) -> RegionalHistoryV1 {
    let selected: BTreeSet<_> = report
        .starting_region
        .settlement_ids
        .iter()
        .copied()
        .collect();
    let retained_events: BTreeSet<_> = report.starting_region.event_ids.iter().copied().collect();
    let populations = report
        .populations
        .iter()
        .filter(|population| selected.contains(&population.location_id))
        .cloned()
        .collect::<Vec<_>>();
    let culture_ids = populations
        .iter()
        .flat_map(|population| population.cultures.iter().map(|share| share.id))
        .chain(report.lore.iter().map(|claim| claim.source_culture_id))
        .collect::<BTreeSet<_>>();
    let faith_ids = populations
        .iter()
        .flat_map(|population| population.faiths.iter().map(|share| share.id))
        .chain(report.lore.iter().filter_map(|claim| claim.source_faith_id))
        .collect::<BTreeSet<_>>();

    RegionalHistoryV1 {
        history_title: report.title.clone(),
        projection_year: report.years,
        starting_region: report.starting_region.clone(),
        populations,
        settlements: report
            .settlements
            .iter()
            .filter(|settlement| selected.contains(&settlement.location_id))
            .cloned()
            .collect(),
        cultures: report
            .cultures
            .iter()
            .filter(|culture| culture_ids.contains(&culture.id))
            .cloned()
            .collect(),
        faiths: report
            .faiths
            .iter()
            .filter(|faith| faith_ids.contains(&faith.id))
            .cloned()
            .collect(),
        institutions: report
            .institutions
            .iter()
            .filter(|institution| {
                selected.contains(&institution.location_id)
                    || culture_ids.contains(&institution.culture_id)
            })
            .cloned()
            .collect(),
        lore: report
            .lore
            .iter()
            .filter(|claim| {
                claim
                    .about_events
                    .iter()
                    .any(|event| retained_events.contains(event))
            })
            .cloned()
            .collect(),
        events: report
            .events
            .iter()
            .filter(|event| retained_events.contains(&event.id))
            .cloned()
            .collect(),
        open_route_ids: report.open_route_ids.clone(),
    }
}

/// Runs a detailed household sample and places every event in historical space.
pub fn run_local_history(
    world: &SurfaceWorldV1,
    regional: &RegionalHistoryV1,
    config: LocalHistoryConfigV1,
    seed: u64,
) -> Result<LocalHistoryReportV1, LocalHistoryError> {
    config.validate()?;
    let selected = &regional.starting_region.settlement_ids;
    if selected.len() != usize::from(config.settlement_count) {
        return Err(LocalHistoryError::SettlementCount {
            expected: usize::from(config.settlement_count),
            found: selected.len(),
        });
    }
    let selected_set = selected.iter().copied().collect::<BTreeSet<_>>();
    if selected_set.len() != selected.len() {
        return Err(LocalHistoryError::SettlementCount {
            expected: usize::from(config.settlement_count),
            found: selected_set.len(),
        });
    }
    for location_id in selected {
        if !world
            .places
            .locations
            .iter()
            .any(|location| location.id == *location_id)
        {
            return Err(LocalHistoryError::MissingLocation(location_id.0));
        }
        if !regional
            .populations
            .iter()
            .any(|population| population.location_id == *location_id && population.people > 0)
        {
            return Err(LocalHistoryError::MissingPopulation(location_id.0));
        }
    }
    validate_place_graph(world, regional)?;

    let graph = LocalGraph::new(
        &world.places.routes,
        regional.open_route_ids.iter().copied().collect(),
    );
    let connections =
        selected_connections(selected, &graph, u32::from(config.travel_days_per_cost))?;

    let base = run_years(config.detailed_scenario.clone(), seed, config.years)?;
    let initial_household_ids = base
        .households
        .iter()
        .filter(|household| household.founded_day == 0)
        .map(|household| household.id)
        .collect::<Vec<_>>();
    if initial_household_ids.len() < selected.len() {
        return Err(LocalHistoryError::InsufficientInitialHouseholds {
            required: selected.len(),
            found: initial_household_ids.len(),
        });
    }
    let initial_projection = allocate_initial_households(
        selected,
        &regional.populations,
        &initial_household_ids,
        seed,
    )?;
    let initial_residences = initial_projection.residences;
    let initial_allocations = initial_projection.allocations;

    let projection_population = regional
        .populations
        .iter()
        .map(|population| u64::from(population.people))
        .sum::<u64>();
    let allocated_population = initial_allocations
        .values()
        .flatten()
        .map(|allocation| u64::from(allocation.people))
        .sum::<u64>();
    if projection_population != allocated_population {
        return Err(LocalHistoryError::ProjectionMismatch);
    }

    let people_by_id = base
        .people
        .iter()
        .map(|person| (person.id, person))
        .collect::<BTreeMap<_, _>>();
    let mut living = base
        .people
        .iter()
        .filter(|person| person.birth_day.is_none())
        .map(|person| person.id)
        .collect::<BTreeSet<_>>();
    let mut person_household = BTreeMap::new();
    let mut household_residence = BTreeMap::new();
    let mut household_residence_event = BTreeMap::new();
    let mut contexts = BTreeMap::new();
    let mut decisions = Vec::new();
    let mut old_to_new = BTreeMap::new();
    let mut events = Vec::with_capacity(base.events.len() + base.households.len());
    let mut next_event_id = 1_u64;
    let mut initial_person_location = BTreeMap::new();
    let mut arrivals = BTreeMap::<LocationId, u32>::new();
    let mut departures = BTreeMap::<LocationId, u32>::new();

    for source in &base.events {
        let mut event = source.clone();
        let remapped_causes = source
            .causes
            .iter()
            .filter_map(|cause| old_to_new.get(cause).copied())
            .collect::<Vec<_>>();
        event.id = EventId(next_event_id);
        next_event_id = next_event_id.saturating_add(1);
        event.causes = remapped_causes;

        if let EventPayloadV1::HouseholdFormed {
            household_id,
            member_ids,
            ..
        } = &source.payload
        {
            let household_id = *household_id;
            let traveler_origins = member_ids
                .iter()
                .filter_map(|person_id| {
                    let previous_household = person_household.get(person_id)?;
                    let location = household_residence.get(previous_household)?;
                    Some((*person_id, *previous_household, *location))
                })
                .collect::<Vec<_>>();
            let origins = traveler_origins
                .iter()
                .map(|(_, _, location)| *location)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            let (destination, reason, support, travel_cost, route_ids) =
                if let Some(location) = initial_residences.get(&household_id) {
                    (
                        *location,
                        ResidenceReasonV1::MacroProjection,
                        0,
                        0,
                        Vec::new(),
                    )
                } else {
                    select_residence(
                        selected,
                        &origins,
                        member_ids,
                        &living,
                        &person_household,
                        &household_residence,
                        &people_by_id,
                        &graph,
                        seed,
                        household_id,
                    )?
                };
            event.location = Some(destination);
            push_tag(&mut event.tags, "place");
            old_to_new.insert(source.id, event.id);
            let formation_event = event.id;
            events.push(event);

            let mut decision_causes = vec![formation_event];
            decision_causes.extend(
                traveler_origins
                    .iter()
                    .filter_map(|(_, household_id, _)| household_residence_event.get(household_id))
                    .copied(),
            );
            decision_causes.sort_unstable();
            decision_causes.dedup();
            let travel_days = travel_cost.saturating_mul(u32::from(config.travel_days_per_cost));
            let settlement_event_id = EventId(next_event_id);
            next_event_id = next_event_id.saturating_add(1);
            events.push(WorldEventV1 {
                id: settlement_event_id,
                time: source.time,
                kind: EventKindV1::HouseholdSettled,
                actors: member_ids.clone(),
                location: Some(destination),
                causes: decision_causes.clone(),
                tags: vec![
                    String::from("family"),
                    String::from("household"),
                    String::from("place"),
                    String::from("migration"),
                ],
                payload: EventPayloadV1::HouseholdSettled {
                    household_id,
                    origin_location_ids: origins.clone(),
                    destination_location_id: destination,
                    traveler_ids: member_ids.clone(),
                    route_ids: route_ids.clone(),
                    travel_cost,
                    travel_days,
                    living_kin_support: support,
                    reason,
                },
            });
            household_residence.insert(household_id, destination);
            household_residence_event.insert(household_id, settlement_event_id);

            for (person_id, _, origin) in &traveler_origins {
                if *origin != destination {
                    *departures.entry(*origin).or_default() = departures
                        .get(origin)
                        .copied()
                        .unwrap_or(0)
                        .saturating_add(1);
                    *arrivals.entry(destination).or_default() = arrivals
                        .get(&destination)
                        .copied()
                        .unwrap_or(0)
                        .saturating_add(1);
                }
                person_household.insert(*person_id, household_id);
            }
            for person_id in member_ids {
                person_household.insert(*person_id, household_id);
                if source.time == SimTime::EPOCH {
                    initial_person_location.insert(*person_id, destination);
                }
            }

            let context = household_context(
                household_id,
                destination,
                initial_allocations
                    .get(&household_id)
                    .cloned()
                    .unwrap_or_default(),
                &traveler_origins,
                &contexts,
                regional,
            );
            contexts.insert(household_id, context);
            decisions.push(ResidenceDecisionV1 {
                household_id,
                settled_day: source.time.day(),
                origin_location_ids: origins,
                destination_location_id: destination,
                traveler_ids: member_ids.clone(),
                route_ids,
                travel_cost,
                travel_days,
                living_kin_support: support,
                reason,
                causes: decision_causes,
            });
            continue;
        }

        event.location = event_location(source, &person_household, &household_residence);
        let related_household = event_household(source).or_else(|| {
            source
                .actors
                .iter()
                .find_map(|person| person_household.get(person))
                .copied()
        });
        if let Some(household_id) = related_household
            && let Some(cause) = household_residence_event.get(&household_id)
        {
            event.causes.push(*cause);
            event.causes.sort_unstable();
            event.causes.dedup();
        }
        if event.location.is_some() {
            push_tag(&mut event.tags, "place");
        }
        old_to_new.insert(source.id, event.id);
        let new_event_id = event.id;
        events.push(event);

        match &source.payload {
            EventPayloadV1::PersonBorn {
                person_id,
                household_id,
                ..
            } => {
                living.insert(*person_id);
                person_household.insert(*person_id, *household_id);
            }
            EventPayloadV1::PersonDied { person_id, .. } => {
                living.remove(person_id);
            }
            EventPayloadV1::HouseholdSettled { household_id, .. } => {
                household_residence_event.insert(*household_id, new_event_id);
            }
            _ => {}
        }
    }

    let mut households = base.households.clone();
    for household in &mut households {
        household.residence_id = household_residence.get(&household.id).copied();
        if household.residence_id.is_none() {
            return Err(LocalHistoryError::MissingHouseholdResidence(household.id.0));
        }
    }
    let settlement_records = build_settlement_records(
        world,
        regional,
        &base.people,
        &households,
        &events,
        &initial_person_location,
        &initial_allocations,
        &arrivals,
        &departures,
    );
    let represented_population = settlement_records
        .iter()
        .map(|settlement| u64::from(settlement.represented_population))
        .sum::<u64>();
    if represented_population != projection_population {
        return Err(LocalHistoryError::ProjectionMismatch);
    }
    let births = settlement_records.iter().map(|place| place.births).sum();
    let deaths = settlement_records.iter().map(|place| place.deaths).sum();
    let residence_decisions = decisions
        .iter()
        .filter(|decision| !decision.origin_location_ids.is_empty())
        .count() as u32;
    let household_migrations = decisions
        .iter()
        .filter(|decision| decision_moved(decision))
        .count() as u32;
    let summary = LocalHistorySummaryV1 {
        schema_version: LOCAL_HISTORY_SCHEMA_V1,
        local_history_id: config.id,
        seed,
        projection_year: regional.projection_year,
        elapsed_years: config.years,
        settlements: settlement_records.len(),
        macro_population: projection_population,
        represented_population,
        living_sample_people: base.summary.living_population,
        births,
        deaths,
        residence_decisions,
        household_migrations,
        located_events: events
            .iter()
            .filter(|event| event.location.is_some())
            .count(),
    };
    let household_contexts = contexts.into_values().collect::<Vec<_>>();
    let chronicle = render_local_chronicle(
        &config.title,
        regional,
        &summary,
        &settlement_records,
        &decisions,
        &connections,
    );

    Ok(LocalHistoryReportV1 {
        title: config.title,
        seed,
        simulation_summary: base.summary,
        people: base.people,
        households,
        events,
        household_contexts,
        residence_decisions: decisions,
        connections,
        settlements: settlement_records,
        lore: regional.lore.clone(),
        cultures: regional.cultures.clone(),
        faiths: regional.faiths.clone(),
        institutions: regional.institutions.clone(),
        summary,
        chronicle,
    })
}

fn validate_place_graph(
    world: &SurfaceWorldV1,
    regional: &RegionalHistoryV1,
) -> Result<(), LocalHistoryError> {
    let location_ids = world
        .places
        .locations
        .iter()
        .map(|location| location.id)
        .collect::<BTreeSet<_>>();
    let mut route_ids = BTreeSet::new();
    for route in &world.places.routes {
        if !route_ids.insert(route.id)
            || route.travel_cost == 0
            || route.endpoints[0] == route.endpoints[1]
            || route
                .endpoints
                .iter()
                .any(|location| !location_ids.contains(location))
        {
            return Err(LocalHistoryError::InvalidRoute(route.id.0));
        }
    }
    if let Some(route) = regional
        .open_route_ids
        .iter()
        .find(|route| !route_ids.contains(route))
    {
        return Err(LocalHistoryError::UnknownOpenRoute(route.0));
    }
    Ok(())
}

#[derive(Clone, Debug)]
struct Edge {
    to: LocationId,
    route_id: RouteId,
    cost: u32,
}

#[derive(Clone, Debug)]
struct LocalGraph {
    adjacency: BTreeMap<LocationId, Vec<Edge>>,
}

impl LocalGraph {
    fn new(routes: &[RouteRecordV1], open_route_ids: BTreeSet<RouteId>) -> Self {
        let mut adjacency = BTreeMap::<LocationId, Vec<Edge>>::new();
        for route in routes
            .iter()
            .filter(|route| !route.locked || open_route_ids.contains(&route.id))
        {
            let [first, second] = route.endpoints;
            adjacency.entry(first).or_default().push(Edge {
                to: second,
                route_id: route.id,
                cost: route.travel_cost,
            });
            adjacency.entry(second).or_default().push(Edge {
                to: first,
                route_id: route.id,
                cost: route.travel_cost,
            });
        }
        for edges in adjacency.values_mut() {
            edges.sort_unstable_by_key(|edge| (edge.cost, edge.route_id, edge.to));
        }
        Self { adjacency }
    }

    fn shortest_path(
        &self,
        from: LocationId,
        to: LocationId,
    ) -> Option<(u32, Vec<RouteId>, Vec<LocationId>)> {
        if from == to {
            return Some((0, Vec::new(), vec![from]));
        }
        let mut distances = BTreeMap::from([(from, 0_u32)]);
        let mut previous = BTreeMap::<LocationId, (LocationId, RouteId)>::new();
        let mut queue = BinaryHeap::from([(Reverse(0_u32), Reverse(from))]);

        while let Some((Reverse(distance), Reverse(location))) = queue.pop() {
            if location == to {
                break;
            }
            if distances
                .get(&location)
                .is_some_and(|known| distance > *known)
            {
                continue;
            }
            for edge in self.adjacency.get(&location).into_iter().flatten() {
                let next_distance = distance.saturating_add(edge.cost);
                let replace = match distances.get(&edge.to) {
                    None => true,
                    Some(known) if next_distance < *known => true,
                    Some(known) if next_distance == *known => previous
                        .get(&edge.to)
                        .is_none_or(|prior| (edge.route_id, location) < (prior.1, prior.0)),
                    _ => false,
                };
                if replace {
                    distances.insert(edge.to, next_distance);
                    previous.insert(edge.to, (location, edge.route_id));
                    queue.push((Reverse(next_distance), Reverse(edge.to)));
                }
            }
        }

        let distance = *distances.get(&to)?;
        let mut route_ids = Vec::new();
        let mut path = vec![to];
        let mut cursor = to;
        while cursor != from {
            let (prior, route_id) = *previous.get(&cursor)?;
            route_ids.push(route_id);
            path.push(prior);
            cursor = prior;
        }
        route_ids.reverse();
        path.reverse();
        Some((distance, route_ids, path))
    }
}

fn selected_connections(
    selected: &[LocationId],
    graph: &LocalGraph,
    travel_days_per_cost: u32,
) -> Result<Vec<LocalConnectionV1>, LocalHistoryError> {
    let mut ordered = selected.to_vec();
    ordered.sort_unstable();
    let mut connections = Vec::new();
    for (index, from) in ordered.iter().enumerate() {
        for to in ordered.iter().skip(index + 1) {
            let Some((travel_cost, route_ids, path)) = graph.shortest_path(*from, *to) else {
                return Err(LocalHistoryError::DisconnectedRegion {
                    from: from.0,
                    to: to.0,
                });
            };
            connections.push(LocalConnectionV1 {
                from: *from,
                to: *to,
                travel_cost,
                travel_days: travel_cost.saturating_mul(travel_days_per_cost),
                route_ids,
                path,
            });
        }
    }
    Ok(connections)
}

struct InitialProjection {
    residences: BTreeMap<HouseholdId, LocationId>,
    allocations: BTreeMap<HouseholdId, Vec<PopulationAllocationV1>>,
}

fn allocate_initial_households(
    selected: &[LocationId],
    populations: &[merra_core::PopulationRecordV1],
    household_ids: &[HouseholdId],
    seed: u64,
) -> Result<InitialProjection, LocalHistoryError> {
    let totals = selected
        .iter()
        .map(|location| {
            populations
                .iter()
                .filter(|population| population.location_id == *location)
                .map(|population| u64::from(population.people))
                .sum::<u64>()
        })
        .collect::<Vec<_>>();
    let total_population = totals.iter().sum::<u64>();
    if total_population == 0 {
        return Err(LocalHistoryError::ProjectionMismatch);
    }
    let mut counts = vec![1_usize; selected.len()];
    let remaining = household_ids.len().saturating_sub(selected.len());
    let mut assigned_extra = 0_usize;
    let mut remainders = Vec::new();
    for (index, total) in totals.iter().enumerate() {
        let numerator = total.saturating_mul(remaining as u64);
        let extra = (numerator / total_population) as usize;
        counts[index] = counts[index].saturating_add(extra);
        assigned_extra = assigned_extra.saturating_add(extra);
        remainders.push((
            numerator % total_population,
            tie_rank(seed, HouseholdId(index as u64 + 1), selected[index]),
            index,
        ));
    }
    remainders
        .sort_unstable_by_key(|(remainder, rank, index)| (Reverse(*remainder), *rank, *index));
    for (_, _, index) in remainders
        .into_iter()
        .take(remaining.saturating_sub(assigned_extra))
    {
        counts[index] = counts[index].saturating_add(1);
    }

    let mut residences = BTreeMap::new();
    let mut households_by_location = BTreeMap::<LocationId, Vec<HouseholdId>>::new();
    let mut cursor = 0_usize;
    for (index, location) in selected.iter().enumerate() {
        let end = cursor.saturating_add(counts[index]);
        for household_id in &household_ids[cursor..end] {
            residences.insert(*household_id, *location);
            households_by_location
                .entry(*location)
                .or_default()
                .push(*household_id);
        }
        cursor = end;
    }
    if cursor != household_ids.len() {
        return Err(LocalHistoryError::ProjectionMismatch);
    }

    let mut allocations = household_ids
        .iter()
        .map(|household| (*household, Vec::new()))
        .collect::<BTreeMap<_, _>>();
    for population in populations {
        let Some(households) = households_by_location.get(&population.location_id) else {
            continue;
        };
        let count = households.len() as u32;
        let base = population.people / count;
        let remainder = population.people % count;
        let mut ranked = households
            .iter()
            .copied()
            .map(|household| {
                (
                    tie_rank(seed ^ population.id.0, household, population.location_id),
                    household,
                )
            })
            .collect::<Vec<_>>();
        ranked.sort_unstable();
        let bonus = ranked
            .into_iter()
            .take(remainder as usize)
            .map(|(_, household)| household)
            .collect::<BTreeSet<_>>();
        for household in households {
            let people = base + u32::from(bonus.contains(household));
            if people > 0 {
                allocations
                    .entry(*household)
                    .or_default()
                    .push(PopulationAllocationV1 {
                        population_id: population.id,
                        people,
                    });
            }
        }
    }
    Ok(InitialProjection {
        residences,
        allocations,
    })
}

#[allow(clippy::too_many_arguments)]
fn select_residence(
    selected: &[LocationId],
    origins: &[LocationId],
    members: &[merra_core::PersonId],
    living: &BTreeSet<merra_core::PersonId>,
    person_household: &BTreeMap<merra_core::PersonId, HouseholdId>,
    household_residence: &BTreeMap<HouseholdId, LocationId>,
    people: &BTreeMap<merra_core::PersonId, &merra_core::PersonRecordV1>,
    graph: &LocalGraph,
    seed: u64,
    household_id: HouseholdId,
) -> Result<(LocationId, ResidenceReasonV1, u16, u32, Vec<RouteId>), LocalHistoryError> {
    let kin = close_kin(members, people)
        .into_iter()
        .filter(|person| living.contains(person) && !members.contains(person))
        .collect::<BTreeSet<_>>();
    let support_by_location = kin
        .iter()
        .filter_map(|person| person_household.get(person))
        .filter_map(|household| household_residence.get(household))
        .fold(
            BTreeMap::<LocationId, u16>::new(),
            |mut counts, location| {
                *counts.entry(*location).or_default() =
                    counts.get(location).copied().unwrap_or(0).saturating_add(1);
                counts
            },
        );

    let mut candidates = Vec::new();
    for destination in selected {
        let support = support_by_location.get(destination).copied().unwrap_or(0);
        let mut summed_cost = 0_u32;
        let mut greatest_cost = 0_u32;
        let mut route_ids = BTreeSet::new();
        for origin in origins {
            let Some((cost, routes, _)) = graph.shortest_path(*origin, *destination) else {
                return Err(LocalHistoryError::DisconnectedRegion {
                    from: origin.0,
                    to: destination.0,
                });
            };
            summed_cost = summed_cost.saturating_add(cost);
            greatest_cost = greatest_cost.max(cost);
            route_ids.extend(routes);
        }
        candidates.push((
            *destination,
            support,
            summed_cost,
            greatest_cost,
            route_ids.into_iter().collect::<Vec<_>>(),
            tie_rank(seed, household_id, *destination),
        ));
    }
    let maximum_support = candidates
        .iter()
        .map(|candidate| candidate.1)
        .max()
        .unwrap_or(0);
    let support_candidates = candidates
        .iter()
        .filter(|candidate| candidate.1 == maximum_support)
        .collect::<Vec<_>>();
    let minimum_cost = support_candidates
        .iter()
        .map(|candidate| candidate.2)
        .min()
        .unwrap_or(0);
    let mut finalists = support_candidates
        .into_iter()
        .filter(|candidate| candidate.2 == minimum_cost)
        .collect::<Vec<_>>();
    finalists.sort_unstable_by_key(|candidate| (candidate.5, candidate.0));
    let chosen = finalists
        .first()
        .copied()
        .ok_or(LocalHistoryError::ProjectionMismatch)?;
    let reason = if maximum_support > 0
        && candidates
            .iter()
            .any(|candidate| candidate.1 < maximum_support)
    {
        ResidenceReasonV1::LivingKin
    } else if candidates
        .iter()
        .filter(|candidate| candidate.1 == maximum_support)
        .any(|candidate| candidate.2 > minimum_cost)
    {
        ResidenceReasonV1::ShortestJourney
    } else {
        ResidenceReasonV1::SeededTieBreak
    };
    Ok((chosen.0, reason, chosen.1, chosen.3, chosen.4.clone()))
}

fn close_kin(
    members: &[merra_core::PersonId],
    people: &BTreeMap<merra_core::PersonId, &merra_core::PersonRecordV1>,
) -> BTreeSet<merra_core::PersonId> {
    let mut kin = BTreeSet::new();
    for member_id in members {
        let Some(member) = people.get(member_id) else {
            continue;
        };
        kin.extend(member.parent_ids.iter().copied());
        for candidate in people.values() {
            if candidate.parent_ids.contains(member_id)
                || (!member.parent_ids.is_empty()
                    && candidate
                        .parent_ids
                        .iter()
                        .any(|parent| member.parent_ids.contains(parent)))
            {
                kin.insert(candidate.id);
            }
        }
    }
    kin
}

fn tie_rank(seed: u64, household_id: HouseholdId, location_id: LocationId) -> u64 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&seed_for_domain(seed, RngDomain::Migration));
    hasher.update(&household_id.0.to_le_bytes());
    hasher.update(&location_id.0.to_le_bytes());
    let bytes = hasher.finalize();
    let mut rank = [0_u8; 8];
    rank.copy_from_slice(&bytes.as_bytes()[..8]);
    u64::from_le_bytes(rank)
}

fn household_context(
    household_id: HouseholdId,
    destination: LocationId,
    represented_populations: Vec<PopulationAllocationV1>,
    traveler_origins: &[(merra_core::PersonId, HouseholdId, LocationId)],
    contexts: &BTreeMap<HouseholdId, HouseholdHistoricalContextV1>,
    regional: &RegionalHistoryV1,
) -> HouseholdHistoricalContextV1 {
    let source_contexts = traveler_origins
        .iter()
        .filter_map(|(_, household, _)| contexts.get(household))
        .collect::<Vec<_>>();
    let population_ids = represented_populations
        .iter()
        .map(|allocation| allocation.population_id)
        .collect::<BTreeSet<_>>();
    let mut culture_ids = regional
        .populations
        .iter()
        .filter(|population| population_ids.contains(&population.id))
        .flat_map(|population| population.cultures.iter().map(|share| share.id))
        .chain(
            source_contexts
                .iter()
                .flat_map(|context| context.culture_ids.iter().copied()),
        )
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let mut faith_ids = regional
        .populations
        .iter()
        .filter(|population| population_ids.contains(&population.id))
        .flat_map(|population| population.faiths.iter().map(|share| share.id))
        .chain(
            source_contexts
                .iter()
                .flat_map(|context| context.faith_ids.iter().copied()),
        )
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    culture_ids.sort_unstable();
    faith_ids.sort_unstable();
    let institution_ids = regional
        .institutions
        .iter()
        .filter(|institution| {
            institution.location_id == destination || culture_ids.contains(&institution.culture_id)
        })
        .map(|institution| institution.id)
        .collect::<Vec<_>>();
    let retained_events = regional
        .starting_region
        .event_ids
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let lore_claim_ids = regional
        .lore
        .iter()
        .filter(|claim| {
            culture_ids.contains(&claim.source_culture_id)
                && claim
                    .about_events
                    .iter()
                    .any(|event| retained_events.contains(event))
        })
        .map(|claim| claim.id)
        .collect::<Vec<_>>();
    HouseholdHistoricalContextV1 {
        household_id,
        residence_id: destination,
        represented_populations,
        culture_ids,
        faith_ids,
        institution_ids,
        lore_claim_ids,
    }
}

fn event_household(event: &WorldEventV1) -> Option<HouseholdId> {
    match &event.payload {
        EventPayloadV1::HouseholdFormed { household_id, .. }
        | EventPayloadV1::PartnershipFormed { household_id, .. }
        | EventPayloadV1::PersonBorn { household_id, .. }
        | EventPayloadV1::HouseholdDissolved { household_id, .. }
        | EventPayloadV1::HouseholdSettled { household_id, .. } => Some(*household_id),
        _ => None,
    }
}

fn event_location(
    event: &WorldEventV1,
    person_household: &BTreeMap<merra_core::PersonId, HouseholdId>,
    household_residence: &BTreeMap<HouseholdId, LocationId>,
) -> Option<LocationId> {
    if let Some(household_id) = event_household(event) {
        return household_residence.get(&household_id).copied();
    }
    event
        .actors
        .iter()
        .find_map(|person| person_household.get(person))
        .and_then(|household| household_residence.get(household))
        .copied()
}

#[allow(clippy::too_many_arguments)]
fn build_settlement_records(
    world: &SurfaceWorldV1,
    regional: &RegionalHistoryV1,
    people: &[merra_core::PersonRecordV1],
    households: &[merra_core::HouseholdRecordV1],
    events: &[WorldEventV1],
    initial_person_location: &BTreeMap<merra_core::PersonId, LocationId>,
    initial_allocations: &BTreeMap<HouseholdId, Vec<PopulationAllocationV1>>,
    arrivals: &BTreeMap<LocationId, u32>,
    departures: &BTreeMap<LocationId, u32>,
) -> Vec<LocalSettlementRecordV1> {
    regional
        .starting_region
        .settlement_ids
        .iter()
        .filter_map(|location_id| {
            let location = world
                .places
                .locations
                .iter()
                .find(|location| location.id == *location_id)?;
            let macro_population = regional
                .populations
                .iter()
                .filter(|population| population.location_id == *location_id)
                .map(|population| population.people)
                .sum();
            let represented_population = initial_allocations
                .iter()
                .filter(|(household, _)| {
                    households.iter().any(|record| {
                        record.id == **household
                            && record.founded_day == 0
                            && record.residence_id == Some(*location_id)
                    })
                })
                .flat_map(|(_, allocations)| allocations)
                .map(|allocation| allocation.people)
                .sum();
            let births = events
                .iter()
                .filter(|event| {
                    event.kind == EventKindV1::PersonBorn && event.location == Some(*location_id)
                })
                .count() as u32;
            let deaths = events
                .iter()
                .filter(|event| {
                    event.kind == EventKindV1::PersonDied && event.location == Some(*location_id)
                })
                .count() as u32;
            let final_living_people = people
                .iter()
                .filter(|person| {
                    person.alive
                        && person.household_id.is_some_and(|household_id| {
                            households.iter().any(|household| {
                                household.id == household_id
                                    && household.residence_id == Some(*location_id)
                            })
                        })
                })
                .count() as u32;
            let initial_sample_people = initial_person_location
                .values()
                .filter(|location| **location == *location_id)
                .count() as u32;
            let active_households = households
                .iter()
                .filter(|household| {
                    household.residence_id == Some(*location_id)
                        && household.dissolved_day.is_none()
                })
                .count() as u32;
            let institution_ids = regional
                .institutions
                .iter()
                .filter(|institution| institution.location_id == *location_id)
                .map(|institution| institution.id)
                .collect();
            let historical_event_ids = regional
                .events
                .iter()
                .filter(|event| event.location == Some(*location_id))
                .map(|event| event.id)
                .collect();
            Some(LocalSettlementRecordV1 {
                location_id: *location_id,
                name: location.name.clone(),
                macro_population,
                represented_population,
                initial_sample_people,
                final_living_people,
                births,
                deaths,
                arrivals: arrivals.get(location_id).copied().unwrap_or(0),
                departures: departures.get(location_id).copied().unwrap_or(0),
                active_households,
                institution_ids,
                historical_event_ids,
            })
        })
        .collect()
}

fn render_local_chronicle(
    title: &str,
    regional: &RegionalHistoryV1,
    summary: &LocalHistorySummaryV1,
    settlements: &[LocalSettlementRecordV1],
    decisions: &[ResidenceDecisionV1],
    connections: &[LocalConnectionV1],
) -> String {
    let mut output = format!(
        "# Chronicle: {title}\n\n\
         - Projection: Year {} of `{}`\n\
         - Settlements: {}\n\
         - Aggregate population reconciled: {}\n\
         - Detailed span: {} years\n\
         - Local sample: {} living, {} births, {} deaths\n\
         - Household migrations: {}\n\
         - Located events: {}\n\n\
         ## Five Settlements\n\n",
        regional.projection_year,
        regional.history_title,
        summary.settlements,
        summary.represented_population,
        summary.elapsed_years,
        summary.living_sample_people,
        summary.births,
        summary.deaths,
        summary.household_migrations,
        summary.located_events,
    );
    for settlement in settlements {
        let change =
            i64::from(settlement.final_living_people) - i64::from(settlement.initial_sample_people);
        output.push_str(&format!(
            "- **{}:** {} macro people → {}/{} detailed living ({change:+}); {} births, {} deaths, {} in, {} out.\n",
            settlement.name,
            settlement.macro_population,
            settlement.final_living_people,
            settlement.initial_sample_people,
            settlement.births,
            settlement.deaths,
            settlement.arrivals,
            settlement.departures,
        ));
    }
    let longest = connections
        .iter()
        .max_by_key(|connection| (connection.travel_cost, connection.from, connection.to));
    output.push_str("\n## Roads and Residence\n\n");
    output.push_str(
        "New households choose one home by living-kin support, then shortest road cost, then an isolated seeded tie-break.\n",
    );
    if let Some(connection) = longest {
        output.push_str(&format!(
            "The longest selected-settlement journey costs {} and takes {} days across {} route segment(s).\n",
            connection.travel_cost,
            connection.travel_days,
            connection.route_ids.len(),
        ));
    }
    let moved = decisions
        .iter()
        .filter(|decision| decision_moved(decision))
        .collect::<Vec<_>>();
    let kin_moves = moved
        .iter()
        .filter(|decision| decision.reason == ResidenceReasonV1::LivingKin)
        .count();
    let road_moves = moved
        .iter()
        .filter(|decision| decision.reason == ResidenceReasonV1::ShortestJourney)
        .count();
    let tied_moves = moved
        .iter()
        .filter(|decision| decision.reason == ResidenceReasonV1::SeededTieBreak)
        .count();
    output.push_str(&format!(
        "{kin_moves} residence decisions were led by kin, {road_moves} by distance, and {tied_moves} by seeded ties.\n"
    ));
    if !regional.lore.is_empty() {
        output.push_str("\n## Inherited Claims\n\n");
        for claim in &regional.lore {
            output.push_str(&format!("- **{}:** {}\n", claim.title, claim.text));
        }
    }
    output
}

fn push_tag(tags: &mut Vec<String>, tag: &str) {
    if !tags.iter().any(|candidate| candidate == tag) {
        tags.push(String::from(tag));
    }
}

fn decision_moved(decision: &ResidenceDecisionV1) -> bool {
    !decision.origin_location_ids.is_empty()
        && decision
            .origin_location_ids
            .iter()
            .any(|origin| *origin != decision.destination_location_id)
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use merra_core::{
        HouseholdId, LocationId, PersonId, PersonRecordV1, RouteId, RouteKindV1, RouteRecordV1,
    };

    use super::{LocalGraph, ResidenceReasonV1, select_residence};

    fn person(id: u64, parent_ids: Vec<PersonId>) -> PersonRecordV1 {
        PersonRecordV1 {
            id: PersonId(id),
            name: format!("Person {id}"),
            given_name: format!("Person{id}"),
            surname: String::from("Test"),
            starting_age_years: 18,
            final_age_years: 18,
            alive: true,
            death_day: None,
            birth_day: None,
            parent_ids,
            household_id: None,
            partner_id: None,
            generation: 0,
        }
    }

    fn graph() -> LocalGraph {
        LocalGraph::new(
            &[
                RouteRecordV1 {
                    id: RouteId(1),
                    endpoints: [LocationId(1), LocationId(2)],
                    kind: RouteKindV1::Land,
                    travel_cost: 4,
                    capacity: 10,
                    locked: false,
                    required_capability: None,
                },
                RouteRecordV1 {
                    id: RouteId(2),
                    endpoints: [LocationId(2), LocationId(3)],
                    kind: RouteKindV1::Land,
                    travel_cost: 2,
                    capacity: 10,
                    locked: false,
                    required_capability: None,
                },
            ],
            BTreeSet::new(),
        )
    }

    #[test]
    fn residence_uses_kin_before_distance() -> Result<(), Box<dyn std::error::Error>> {
        let people = [
            person(1, vec![PersonId(3)]),
            person(2, Vec::new()),
            person(3, Vec::new()),
        ];
        let people_by_id = people
            .iter()
            .map(|person| (person.id, person))
            .collect::<BTreeMap<_, _>>();
        let living = people.iter().map(|person| person.id).collect();
        let person_household = BTreeMap::from([(PersonId(3), HouseholdId(9))]);
        let household_residence = BTreeMap::from([(HouseholdId(9), LocationId(3))]);

        let choice = select_residence(
            &[LocationId(1), LocationId(2), LocationId(3)],
            &[LocationId(1)],
            &[PersonId(1), PersonId(2)],
            &living,
            &person_household,
            &household_residence,
            &people_by_id,
            &graph(),
            42,
            HouseholdId(10),
        )?;

        assert_eq!(choice.0, LocationId(3));
        assert_eq!(choice.1, ResidenceReasonV1::LivingKin);
        assert_eq!(choice.2, 1);
        assert_eq!(choice.3, 6);
        Ok(())
    }

    #[test]
    fn residence_falls_back_to_roads_then_seeded_rank() -> Result<(), Box<dyn std::error::Error>> {
        let people = [person(1, Vec::new()), person(2, Vec::new())];
        let people_by_id = people
            .iter()
            .map(|person| (person.id, person))
            .collect::<BTreeMap<_, _>>();
        let living = people.iter().map(|person| person.id).collect();
        let by_road = select_residence(
            &[LocationId(1), LocationId(2), LocationId(3)],
            &[LocationId(1)],
            &[PersonId(1), PersonId(2)],
            &living,
            &BTreeMap::new(),
            &BTreeMap::new(),
            &people_by_id,
            &graph(),
            42,
            HouseholdId(10),
        )?;
        assert_eq!(by_road.0, LocationId(1));
        assert_eq!(by_road.1, ResidenceReasonV1::ShortestJourney);

        let tied_first = select_residence(
            &[LocationId(1), LocationId(2)],
            &[LocationId(1), LocationId(2)],
            &[PersonId(1), PersonId(2)],
            &living,
            &BTreeMap::new(),
            &BTreeMap::new(),
            &people_by_id,
            &graph(),
            42,
            HouseholdId(11),
        )?;
        let tied_second = select_residence(
            &[LocationId(1), LocationId(2)],
            &[LocationId(1), LocationId(2)],
            &[PersonId(1), PersonId(2)],
            &living,
            &BTreeMap::new(),
            &BTreeMap::new(),
            &people_by_id,
            &graph(),
            42,
            HouseholdId(11),
        )?;
        assert_eq!(tied_first, tied_second);
        assert_eq!(tied_first.1, ResidenceReasonV1::SeededTieBreak);
        Ok(())
    }
}
