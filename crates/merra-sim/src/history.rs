//! Bevy-scheduled aggregate history over a setting-independent place graph.

use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, VecDeque},
};

use bevy_app::App;
use bevy_ecs::{
    prelude::{ResMut, Resource},
    schedule::{Schedule, ScheduleLabel},
};
use merra_core::{
    AffiliationShareV1, CultureId, CultureRecordV1, EventId, FaithId, FaithRecordV1, FaithSeedV1,
    FeatureKindV1, FounderSeedV1, HISTORY_SCHEMA_V1, HistoricalEventKindV1,
    HistoricalEventPayloadV1, HistoricalEventV1, HistoricalSubjectV1, HistoryConfigV1,
    HistoryError, HistorySummaryV1, ImportantPlaceV1, InstitutionId, InstitutionRecordV1,
    LineageDefinitionV1, LineageId, LocationId, LoreClaimV1, PlaceGraphV1, PolityId,
    PolityRecordV1, PopulationId, PopulationRecordV1, RouteId, RouteKindV1, SettlementRecordV1,
    SimTime, StartingRegionV1, SurfaceWorldV1, WorldFeatureV1,
};
use thiserror::Error;

/// Deterministic macro-history failure.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum HistorySimulationError {
    #[error(transparent)]
    Config(#[from] HistoryError),
    #[error("historical simulation has already completed")]
    AlreadyFinished,
    #[error("generated world is missing a configured mythic motif")]
    MissingMythicSource,
}

/// Complete inspectable evidence from one aggregate historical age.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HistoricalReport {
    pub title: String,
    pub seed: u64,
    pub years: u32,
    pub events: Vec<HistoricalEventV1>,
    pub populations: Vec<PopulationRecordV1>,
    pub settlements: Vec<SettlementRecordV1>,
    pub cultures: Vec<CultureRecordV1>,
    pub faiths: Vec<FaithRecordV1>,
    pub institutions: Vec<InstitutionRecordV1>,
    pub polities: Vec<PolityRecordV1>,
    pub lore: Vec<LoreClaimV1>,
    pub important_places: Vec<ImportantPlaceV1>,
    pub starting_region: StartingRegionV1,
    pub summary: HistorySummaryV1,
    pub chronicle: String,
    pub open_route_ids: Vec<RouteId>,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq, ScheduleLabel)]
struct HistoricalYear;

#[derive(Resource)]
struct MacroState {
    config: HistoryConfigV1,
    graph: PlaceGraphV1,
    lineages: Vec<LineageDefinitionV1>,
    seed: u64,
    year: u32,
    next_event: u64,
    next_population: u64,
    next_culture: u64,
    next_faith: u64,
    next_institution: u64,
    next_polity: u64,
    events: Vec<HistoricalEventV1>,
    populations: Vec<PopulationRecordV1>,
    settlements: Vec<SettlementRecordV1>,
    cultures: Vec<CultureRecordV1>,
    faiths: Vec<FaithRecordV1>,
    institutions: Vec<InstitutionRecordV1>,
    polities: Vec<PolityRecordV1>,
    open_routes: BTreeSet<RouteId>,
    first_contact_year: Option<u32>,
    first_contact_event: Option<EventId>,
    first_contact_location: Option<LocationId>,
    history_completed: bool,
}

/// A coarse historical simulation whose annual transition is a Bevy schedule.
pub struct HistoricalSimulation {
    app: App,
    title: String,
    years: u32,
    seed: u64,
    finished: bool,
}

impl HistoricalSimulation {
    /// Seeds data-defined populations into configured homeland classes.
    pub fn from_world(
        world: &SurfaceWorldV1,
        config: HistoryConfigV1,
        seed: u64,
    ) -> Result<Self, HistorySimulationError> {
        Self::from_place_graph(world.places.clone(), &world.features, config, seed)
    }

    /// Seeds history from a generic graph with no surface-world dependency.
    pub fn from_place_graph(
        graph: PlaceGraphV1,
        features: &[WorldFeatureV1],
        config: HistoryConfigV1,
        seed: u64,
    ) -> Result<Self, HistorySimulationError> {
        config.validate()?;
        let state = MacroState::new(graph, features, config.clone(), seed)?;
        let mut app = App::new();
        app.insert_resource(state);
        let mut schedule = Schedule::new(HistoricalYear);
        schedule.add_systems(advance_history_year);
        app.add_schedule(schedule);
        Ok(Self {
            app,
            title: config.title,
            years: config.years,
            seed,
            finished: false,
        })
    }

    /// Advances the requested number of aggregate years.
    pub fn advance(&mut self, years: u32) -> Result<(), HistorySimulationError> {
        if self.finished {
            return Err(HistorySimulationError::AlreadyFinished);
        }
        for _ in 0..years {
            self.app.world_mut().run_schedule(HistoricalYear);
        }
        Ok(())
    }

    /// Completes the historical age and emits its final event.
    pub fn finish(&mut self) -> Result<(), HistorySimulationError> {
        if self.finished {
            return Err(HistorySimulationError::AlreadyFinished);
        }
        let mut state = self.app.world_mut().resource_mut::<MacroState>();
        state.finish();
        self.finished = true;
        Ok(())
    }

    /// Builds immutable machine and human evidence.
    #[must_use]
    pub fn report(&self) -> HistoricalReport {
        let state = self.app.world().resource::<MacroState>();
        state.report(self.title.clone(), self.seed, self.years)
    }
}

/// Runs a complete historical age.
pub fn run_history(
    world: &SurfaceWorldV1,
    config: HistoryConfigV1,
    seed: u64,
) -> Result<HistoricalReport, HistorySimulationError> {
    let years = config.years;
    let mut simulation = HistoricalSimulation::from_world(world, config, seed)?;
    simulation.advance(years)?;
    simulation.finish()?;
    Ok(simulation.report())
}

/// Runs history over a hand-authored or non-surface place graph.
pub fn run_history_on_graph(
    graph: PlaceGraphV1,
    features: &[WorldFeatureV1],
    config: HistoryConfigV1,
    seed: u64,
) -> Result<HistoricalReport, HistorySimulationError> {
    let years = config.years;
    let mut simulation = HistoricalSimulation::from_place_graph(graph, features, config, seed)?;
    simulation.advance(years)?;
    simulation.finish()?;
    Ok(simulation.report())
}

fn advance_history_year(mut state: ResMut<MacroState>) {
    state.year = state.year.saturating_add(1);
    state.grow_populations();
    state.expand_settlements();
    state.form_scheduled_faiths();
    state.form_institutions();
    state.form_polities();
    state.open_sea_and_contact();
    state.schism_after_contact();
}

impl MacroState {
    fn new(
        graph: PlaceGraphV1,
        features: &[WorldFeatureV1],
        config: HistoryConfigV1,
        seed: u64,
    ) -> Result<Self, HistorySimulationError> {
        let mut state = Self {
            lineages: config.lineages.clone(),
            config,
            graph,
            seed,
            year: 0,
            next_event: 1,
            next_population: 1,
            next_culture: 1,
            next_faith: 1,
            next_institution: 1,
            next_polity: 1,
            events: Vec::new(),
            populations: Vec::new(),
            settlements: Vec::new(),
            cultures: Vec::new(),
            faiths: Vec::new(),
            institutions: Vec::new(),
            polities: Vec::new(),
            open_routes: BTreeSet::new(),
            first_contact_year: None,
            first_contact_event: None,
            first_contact_location: None,
            history_completed: false,
        };
        state.open_routes = state
            .graph
            .routes
            .iter()
            .filter(|route| !route.locked)
            .map(|route| route.id)
            .collect();
        let started = state.push_event(
            HistoricalEventKindV1::HistoryStarted,
            Vec::new(),
            None,
            Vec::new(),
            vec![String::from("history")],
            HistoricalEventPayloadV1::HistoryStarted {
                history_id: state.config.id.clone(),
                seed,
            },
        );
        state.seed_populations(features, started)?;
        Ok(state)
    }

    fn seed_populations(
        &mut self,
        features: &[WorldFeatureV1],
        started: EventId,
    ) -> Result<(), HistorySimulationError> {
        let founders = self.config.founders.clone();
        let selected = select_founder_locations(&founders, &self.graph)?;
        let mut culture_ids = Vec::new();
        for founder in &founders {
            let culture_seed = founder.culture.clone();
            let culture_id = CultureId(self.next_culture);
            self.next_culture = self.next_culture.saturating_add(1);
            let event = self.push_event(
                HistoricalEventKindV1::CultureFounded,
                vec![HistoricalSubjectV1::Culture(culture_id)],
                None,
                vec![started],
                vec![String::from("culture")],
                HistoricalEventPayloadV1::CultureFounded {
                    culture_id,
                    name: culture_seed.name.clone(),
                },
            );
            self.cultures.push(CultureRecordV1 {
                id: culture_id,
                key: culture_seed.key,
                name: culture_seed.name,
                founded_year: 0,
                origin_event: event,
                ritual_days_per_year: culture_seed.ritual_days_per_year,
                sacred_contribution_per_10_000: culture_seed.sacred_contribution_per_10_000,
                institutional_preservation_per_10_000: culture_seed
                    .institutional_preservation_per_10_000,
                faith_transmission_per_10_000: culture_seed.faith_transmission_per_10_000,
            });
            culture_ids.push(culture_id);
        }

        for (culture_index, (founder, location_id)) in founders.iter().zip(selected).enumerate() {
            let location = self.location(location_id).clone();
            let settlement_event = self.push_event(
                HistoricalEventKindV1::SettlementFounded,
                vec![HistoricalSubjectV1::Location(location_id)],
                Some(location_id),
                vec![started],
                vec![String::from("settlement")],
                HistoricalEventPayloadV1::SettlementFounded {
                    location_id,
                    name: location.name.clone(),
                },
            );
            self.settlements.push(SettlementRecordV1 {
                location_id,
                name: location.name.clone(),
                founded_year: 0,
                abandoned_year: None,
                population: self.config.initial_population_per_cohort,
                founding_event: settlement_event,
            });
            let population_id = PopulationId(self.next_population);
            self.next_population = self.next_population.saturating_add(1);
            let population_event = self.push_event(
                HistoricalEventKindV1::PopulationSeeded,
                vec![
                    HistoricalSubjectV1::Population(population_id),
                    HistoricalSubjectV1::Location(location_id),
                    HistoricalSubjectV1::Culture(culture_ids[culture_index]),
                ],
                Some(location_id),
                vec![settlement_event],
                vec![String::from("population")],
                HistoricalEventPayloadV1::PopulationSeeded {
                    population_id,
                    people: self.config.initial_population_per_cohort,
                },
            );
            self.populations.push(PopulationRecordV1 {
                id: population_id,
                name: format!("People of {}", location.name),
                location_id,
                people: self.config.initial_population_per_cohort,
                founded_year: 0,
                lineage: vec![AffiliationShareV1 {
                    id: founder.lineage_id,
                    parts_per_10_000: 10_000,
                }],
                cultures: vec![AffiliationShareV1 {
                    id: culture_ids[culture_index],
                    parts_per_10_000: 10_000,
                }],
                faiths: Vec::new(),
            });
            let _ = population_event;
        }

        let founding_faiths = self
            .config
            .faiths
            .iter()
            .filter(|faith| faith.founded_year == 0)
            .cloned()
            .collect::<Vec<_>>();
        for faith in founding_faiths {
            self.found_faith(&faith, features, started)?;
        }
        Ok(())
    }

    fn found_faith(
        &mut self,
        seed: &FaithSeedV1,
        features: &[WorldFeatureV1],
        cause: EventId,
    ) -> Result<(), HistorySimulationError> {
        let culture = self
            .cultures
            .iter()
            .find(|culture| culture.key == seed.culture_key)
            .cloned()
            .ok_or(HistoryError::InvalidFaith)?;
        let source_feature = if let Some(motif_key) = &seed.source_motif_id {
            Some(
                features
                    .iter()
                    .find(|feature| {
                        matches!(
                            &feature.kind,
                            FeatureKindV1::MythicTrace { motif_id } if motif_id == motif_key
                        )
                    })
                    .or_else(|| {
                        features.iter().find(|feature| {
                            matches!(feature.kind, FeatureKindV1::MythicTrace { .. })
                        })
                    })
                    .ok_or(HistorySimulationError::MissingMythicSource)?,
            )
        } else {
            None
        };
        let faith_id = FaithId(self.next_faith);
        self.next_faith = self.next_faith.saturating_add(1);
        let mut subjects = vec![
            HistoricalSubjectV1::Faith(faith_id),
            HistoricalSubjectV1::Culture(culture.id),
        ];
        if let Some(feature) = source_feature {
            subjects.push(HistoricalSubjectV1::Feature(feature.id));
        }
        let mut tags = vec![String::from("faith")];
        tags.extend(seed.tags.iter().cloned());
        let faith_event = self.push_event(
            HistoricalEventKindV1::FaithFounded,
            subjects,
            None,
            vec![cause],
            tags,
            HistoricalEventPayloadV1::FaithFounded {
                faith_id,
                name: seed.name.clone(),
            },
        );
        self.faiths.push(FaithRecordV1 {
            id: faith_id,
            key: seed.key.clone(),
            name: seed.name.clone(),
            founded_year: self.year,
            origin_event: faith_event,
            source_feature_id: source_feature.map(|feature| feature.id),
            parent_faith_id: None,
        });
        for population in &mut self.populations {
            if population
                .cultures
                .iter()
                .any(|share| share.id == culture.id)
            {
                population.faiths = vec![AffiliationShareV1 {
                    id: faith_id,
                    parts_per_10_000: 10_000,
                }];
            }
        }
        if seed.founding_institution {
            let location = self
                .populations
                .iter()
                .find(|population| {
                    population
                        .cultures
                        .iter()
                        .any(|share| share.id == culture.id)
                })
                .map_or(LocationId(1), |population| population.location_id);
            self.create_institution(
                format!("Keepers of {}", seed.name),
                culture.id,
                Some(faith_id),
                location,
                faith_event,
            );
        }
        Ok(())
    }

    fn grow_populations(&mut self) {
        let locations: BTreeMap<_, _> = self
            .graph
            .locations
            .iter()
            .map(|location| (location.id, location.clone()))
            .collect();
        let lineages: BTreeMap<_, _> = self
            .lineages
            .iter()
            .map(|lineage| (lineage.id, lineage.clone()))
            .collect();
        for population in &mut self.populations {
            let Some(location) = locations.get(&population.location_id) else {
                continue;
            };
            let mortality = weighted_physiology(&population.lineage, &lineages, |lineage| {
                lineage.physiology.adult_mortality_multiplier_per_10_000
            });
            let power = weighted_physiology(&population.lineage, &lineages, |lineage| {
                lineage.physiology.physical_power_per_10_000
            });
            let sustenance = weighted_physiology(&population.lineage, &lineages, |lineage| {
                lineage.physiology.sustenance_demand_per_10_000
            })
            .max(1);
            let effective_capacity = u64::from(location.carrying_capacity).saturating_mul(10_000)
                / u64::from(sustenance);
            let infrastructure_bonus = u64::from(power.saturating_sub(10_000)) / 5;
            let capacity = effective_capacity
                .saturating_add(infrastructure_bonus)
                .max(1);
            let births_per_10_000 = 245_u64;
            let deaths_per_10_000 = 125_u64.saturating_mul(u64::from(mortality)) / 10_000;
            let hazard = u64::from(location.hazard_per_10_000) / 20;
            let base_growth = births_per_10_000
                .saturating_sub(deaths_per_10_000)
                .saturating_sub(hazard);
            let current = u64::from(population.people);
            let pressure = if current > capacity {
                (current - capacity).saturating_mul(400) / capacity
            } else {
                0
            };
            let growth_rate = base_growth.saturating_sub(pressure.min(base_growth));
            let growth = current.saturating_mul(growth_rate) / 10_000;
            population.people = u32::try_from(current.saturating_add(growth))
                .unwrap_or(u32::MAX)
                .max(1);
        }
        self.refresh_settlement_populations();
    }

    fn expand_settlements(&mut self) {
        if self.year == 0 || !self.year.is_multiple_of(30) {
            return;
        }
        let occupied: BTreeSet<_> = self
            .populations
            .iter()
            .map(|population| population.location_id)
            .collect();
        let mut sources: Vec<_> = self
            .populations
            .iter()
            .enumerate()
            .filter(|(_, population)| population.people >= 900)
            .map(|(index, population)| (Reverse(population.people), population.id, index))
            .collect();
        sources.sort_unstable();
        for (_, _, source_index) in sources {
            let source_location = self.populations[source_index].location_id;
            let target = self
                .graph
                .routes
                .iter()
                .filter(|route| self.open_routes.contains(&route.id))
                .filter_map(|route| other_endpoint(route.endpoints, source_location))
                .filter(|location| !occupied.contains(location))
                .min();
            let Some(target) = target else {
                continue;
            };
            self.migrate_to_empty(source_index, target);
            break;
        }
    }

    fn migrate_to_empty(&mut self, source_index: usize, target: LocationId) {
        let migrants = (self.populations[source_index].people / 5).max(100);
        self.populations[source_index].people = self.populations[source_index]
            .people
            .saturating_sub(migrants);
        let source = self.populations[source_index].clone();
        let location = self.location(target).clone();
        let settlement_event = self.push_event(
            HistoricalEventKindV1::SettlementFounded,
            vec![HistoricalSubjectV1::Location(target)],
            Some(target),
            self.last_event().into_iter().collect(),
            vec![String::from("settlement"), String::from("migration")],
            HistoricalEventPayloadV1::SettlementFounded {
                location_id: target,
                name: location.name.clone(),
            },
        );
        self.settlements.push(SettlementRecordV1 {
            location_id: target,
            name: location.name.clone(),
            founded_year: self.year,
            abandoned_year: None,
            population: migrants,
            founding_event: settlement_event,
        });
        let population_id = PopulationId(self.next_population);
        self.next_population = self.next_population.saturating_add(1);
        self.push_event(
            HistoricalEventKindV1::PopulationMigrated,
            vec![
                HistoricalSubjectV1::Population(population_id),
                HistoricalSubjectV1::Location(source.location_id),
                HistoricalSubjectV1::Location(target),
            ],
            Some(target),
            vec![settlement_event],
            vec![String::from("migration")],
            HistoricalEventPayloadV1::PopulationMigrated {
                population_id,
                from: source.location_id,
                to: target,
                people: migrants,
            },
        );
        self.populations.push(PopulationRecordV1 {
            id: population_id,
            name: format!("People of {}", location.name),
            location_id: target,
            people: migrants,
            founded_year: self.year,
            lineage: source.lineage,
            cultures: source.cultures,
            faiths: source.faiths,
        });
        self.refresh_settlement_populations();
    }

    fn form_scheduled_faiths(&mut self) {
        let scheduled = self
            .config
            .faiths
            .iter()
            .filter(|faith| {
                faith.founded_year == self.year
                    && !self.faiths.iter().any(|existing| existing.key == faith.key)
            })
            .cloned()
            .collect::<Vec<_>>();
        for faith in scheduled {
            let cause = self.last_event().unwrap_or(EventId(1));
            let result = self.found_faith(&faith, &[], cause);
            debug_assert!(result.is_ok(), "validated scheduled faith must resolve");
        }
    }

    fn form_institutions(&mut self) {
        if self.year == 0 || !self.year.is_multiple_of(100) {
            return;
        }
        let Some(population) = self
            .populations
            .iter()
            .max_by_key(|population| (population.people, Reverse(population.id)))
            .cloned()
        else {
            return;
        };
        let culture_id = population
            .cultures
            .iter()
            .max_by_key(|share| share.parts_per_10_000)
            .map_or(CultureId(1), |share| share.id);
        let faith_id = population
            .faiths
            .iter()
            .max_by_key(|share| share.parts_per_10_000)
            .map(|share| share.id);
        let location_name = self.location(population.location_id).name.clone();
        self.create_institution(
            format!("{} Assembly", location_name),
            culture_id,
            faith_id,
            population.location_id,
            self.last_event().unwrap_or(EventId(1)),
        );
    }

    fn create_institution(
        &mut self,
        name: String,
        culture_id: CultureId,
        faith_id: Option<FaithId>,
        location_id: LocationId,
        cause: EventId,
    ) {
        let institution_id = InstitutionId(self.next_institution);
        self.next_institution = self.next_institution.saturating_add(1);
        let event = self.push_event(
            HistoricalEventKindV1::InstitutionFounded,
            vec![
                HistoricalSubjectV1::Institution(institution_id),
                HistoricalSubjectV1::Culture(culture_id),
            ],
            Some(location_id),
            vec![cause],
            vec![String::from("institution")],
            HistoricalEventPayloadV1::InstitutionFounded {
                institution_id,
                name: name.clone(),
            },
        );
        self.institutions.push(InstitutionRecordV1 {
            id: institution_id,
            name,
            founded_year: self.year,
            dissolved_year: None,
            culture_id,
            faith_id,
            location_id,
            founding_event: event,
        });
    }

    fn form_polities(&mut self) {
        if !matches!(self.year, 200 | 400) {
            return;
        }
        let location_ids = self
            .settlements
            .iter()
            .filter(|settlement| settlement.abandoned_year.is_none())
            .take(8)
            .map(|settlement| settlement.location_id)
            .collect::<Vec<_>>();
        let culture_ids = self
            .cultures
            .iter()
            .take(3)
            .map(|culture| culture.id)
            .collect::<Vec<_>>();
        if location_ids.is_empty() {
            return;
        }
        let polity_id = PolityId(self.next_polity);
        self.next_polity = self.next_polity.saturating_add(1);
        let name = if self.year == 200 {
            String::from("The River Compact")
        } else {
            String::from("The Concord of Shores")
        };
        let event = self.push_event(
            HistoricalEventKindV1::PolityFounded,
            vec![HistoricalSubjectV1::Polity(polity_id)],
            location_ids.first().copied(),
            self.last_event().into_iter().collect(),
            vec![String::from("polity")],
            HistoricalEventPayloadV1::PolityFounded {
                polity_id,
                name: name.clone(),
            },
        );
        self.polities.push(PolityRecordV1 {
            id: polity_id,
            name,
            founded_year: self.year,
            dissolved_year: None,
            culture_ids,
            location_ids,
            founding_event: event,
        });
    }

    fn open_sea_and_contact(&mut self) {
        if self.first_contact_year.is_some() {
            return;
        }
        let Some(route) = self
            .graph
            .routes
            .iter()
            .find(|route| {
                route.locked
                    && route.required_capability.as_deref() == Some("navigation")
                    && !self.open_routes.contains(&route.id)
            })
            .cloned()
        else {
            return;
        };
        let navigation_score = self
            .year
            .saturating_add(self.institutions.len() as u32 * 18)
            .saturating_add(self.settlements.len() as u32 * 4)
            .saturating_add(
                u32::try_from(
                    self.populations
                        .iter()
                        .map(|population| u64::from(population.people))
                        .sum::<u64>()
                        / 5_000,
                )
                .unwrap_or(u32::MAX),
            );
        let variance = history_roll(self.seed, self.year, 0x5345_4120) % 80;
        if navigation_score.saturating_add(variance) < self.config.contact_navigation_threshold {
            return;
        }
        self.open_routes.insert(route.id);
        let (kind, payload) = if route.kind == RouteKindV1::Sea {
            (
                HistoricalEventKindV1::SeaRouteOpened,
                HistoricalEventPayloadV1::SeaRouteOpened { route_id: route.id },
            )
        } else {
            (
                HistoricalEventKindV1::RouteOpened,
                HistoricalEventPayloadV1::RouteOpened {
                    route_id: route.id,
                    capability: String::from("navigation"),
                },
            )
        };
        let opened = self.push_event(
            kind,
            vec![
                HistoricalSubjectV1::Location(route.endpoints[0]),
                HistoricalSubjectV1::Location(route.endpoints[1]),
            ],
            None,
            self.institutions
                .iter()
                .rev()
                .take(2)
                .map(|institution| institution.founding_event)
                .collect(),
            vec![
                String::from("navigation"),
                format!("{:?}", route.kind).to_lowercase(),
            ],
            payload,
        );
        self.create_first_contact(route, opened);
    }

    fn create_first_contact(&mut self, route: merra_core::RouteRecordV1, opened: EventId) {
        let mut route_lineages = route
            .endpoints
            .iter()
            .filter_map(|endpoint| {
                let location = self.location(*endpoint);
                self.config
                    .founders
                    .iter()
                    .enumerate()
                    .find(|(_, founder)| {
                        location.tags.iter().any(|tag| tag == &founder.homeland_tag)
                    })
                    .map(|(index, founder)| (index, founder.lineage_id))
            })
            .collect::<Vec<_>>();
        route_lineages.sort_unstable();
        route_lineages.dedup_by_key(|(_, lineage)| *lineage);
        let route_lineages = route_lineages
            .into_iter()
            .map(|(_, lineage)| lineage)
            .collect::<Vec<_>>();
        let [first_lineage, second_lineage, ..] = route_lineages.as_slice() else {
            return;
        };
        let first_index = self
            .populations
            .iter()
            .enumerate()
            .filter(|(_, population)| lineage_share(population, *first_lineage) >= 5_000)
            .max_by_key(|(_, population)| population.people)
            .map(|(index, _)| index);
        let second_index = self
            .populations
            .iter()
            .enumerate()
            .filter(|(_, population)| lineage_share(population, *second_lineage) >= 5_000)
            .max_by_key(|(_, population)| population.people)
            .map(|(index, _)| index);
        let (Some(first_index), Some(second_index)) = (first_index, second_index) else {
            return;
        };
        let first = self.populations[first_index].clone();
        let second = self.populations[second_index].clone();
        let first_homelands = self
            .config
            .founders
            .iter()
            .filter(|founder| founder.lineage_id == *first_lineage)
            .map(|founder| founder.homeland_tag.as_str())
            .collect::<BTreeSet<_>>();
        let location = route
            .endpoints
            .iter()
            .copied()
            .find(|endpoint| {
                self.location(*endpoint)
                    .tags
                    .iter()
                    .any(|tag| first_homelands.contains(tag.as_str()))
            })
            .unwrap_or(route.endpoints[0]);
        let first_people = (first.people / 10).max(50);
        let second_people = (second.people / 10).max(50);
        self.populations[first_index].people = self.populations[first_index]
            .people
            .saturating_sub(first_people);
        self.populations[second_index].people = self.populations[second_index]
            .people
            .saturating_sub(second_people);
        let population_id = PopulationId(self.next_population);
        self.next_population = self.next_population.saturating_add(1);
        let contact = self.push_event(
            HistoricalEventKindV1::FirstContact,
            vec![
                HistoricalSubjectV1::Population(first.id),
                HistoricalSubjectV1::Population(second.id),
                HistoricalSubjectV1::Location(location),
            ],
            Some(location),
            vec![opened],
            vec![
                String::from("first-contact"),
                format!("{:?}", route.kind).to_lowercase(),
            ],
            HistoricalEventPayloadV1::FirstContact {
                populations: [first.id, second.id],
            },
        );
        let lineage =
            combine_affiliations(&first.lineage, first_people, &second.lineage, second_people);
        let mut cultures = combine_affiliations(
            &first.cultures,
            first_people,
            &second.cultures,
            second_people,
        );
        let contact_culture = self.create_contact_culture(contact);
        cultures = add_affiliation(cultures, contact_culture, 2_000);
        let faiths =
            combine_affiliations(&first.faiths, first_people, &second.faiths, second_people);
        let contact_culture_name = self.config.contact_culture.name.clone();
        self.populations.push(PopulationRecordV1 {
            id: population_id,
            name: format!("{contact_culture_name} of {}", self.location(location).name),
            location_id: location,
            people: first_people.saturating_add(second_people),
            founded_year: self.year,
            lineage: lineage.clone(),
            cultures,
            faiths: faiths.clone(),
        });
        let mixed = self.push_event(
            HistoricalEventKindV1::PopulationsMixed,
            vec![
                HistoricalSubjectV1::Population(population_id),
                HistoricalSubjectV1::Location(location),
            ],
            Some(location),
            vec![contact],
            vec![String::from("mixed-lineage"), String::from("migration")],
            HistoricalEventPayloadV1::PopulationsMixed {
                location_id: location,
                lineages: lineage,
            },
        );
        if let Some(faith) = faiths
            .iter()
            .max_by_key(|share| share.parts_per_10_000)
            .map(|share| share.id)
        {
            self.push_event(
                HistoricalEventKindV1::FaithSpread,
                vec![
                    HistoricalSubjectV1::Faith(faith),
                    HistoricalSubjectV1::Population(population_id),
                ],
                Some(location),
                vec![mixed],
                vec![String::from("faith"), String::from("contact")],
                HistoricalEventPayloadV1::FaithSpread {
                    faith_id: faith,
                    population_id,
                },
            );
        }
        if !self
            .settlements
            .iter()
            .any(|settlement| settlement.location_id == location)
        {
            let name = self.location(location).name.clone();
            self.settlements.push(SettlementRecordV1 {
                location_id: location,
                name,
                founded_year: self.year,
                abandoned_year: None,
                population: first_people.saturating_add(second_people),
                founding_event: contact,
            });
        }
        self.first_contact_year = Some(self.year);
        self.first_contact_event = Some(contact);
        self.first_contact_location = Some(location);
        self.refresh_settlement_populations();
    }

    fn create_contact_culture(&mut self, cause: EventId) -> CultureId {
        let seed = self.config.contact_culture.clone();
        let culture_id = CultureId(self.next_culture);
        self.next_culture = self.next_culture.saturating_add(1);
        let event = self.push_event(
            HistoricalEventKindV1::CultureFounded,
            vec![HistoricalSubjectV1::Culture(culture_id)],
            self.first_contact_location,
            vec![cause],
            vec![String::from("culture"), String::from("contact")],
            HistoricalEventPayloadV1::CultureFounded {
                culture_id,
                name: seed.name.clone(),
            },
        );
        self.cultures.push(CultureRecordV1 {
            id: culture_id,
            key: seed.key,
            name: seed.name,
            founded_year: self.year,
            origin_event: event,
            ritual_days_per_year: seed.ritual_days_per_year,
            sacred_contribution_per_10_000: seed.sacred_contribution_per_10_000,
            institutional_preservation_per_10_000: seed.institutional_preservation_per_10_000,
            faith_transmission_per_10_000: seed.faith_transmission_per_10_000,
        });
        culture_id
    }

    fn schism_after_contact(&mut self) {
        let Some(contact_year) = self.first_contact_year else {
            return;
        };
        if self.year != contact_year.saturating_add(80)
            || self
                .faiths
                .iter()
                .any(|faith| faith.parent_faith_id.is_some())
        {
            return;
        }
        let Some(parent) = self.faiths.first().cloned() else {
            return;
        };
        let child_id = FaithId(self.next_faith);
        self.next_faith = self.next_faith.saturating_add(1);
        let name = format!("Open Hand of {}", parent.name);
        let event = self.push_event(
            HistoricalEventKindV1::FaithSchism,
            vec![
                HistoricalSubjectV1::Faith(parent.id),
                HistoricalSubjectV1::Faith(child_id),
            ],
            self.first_contact_location,
            self.first_contact_event.into_iter().collect(),
            vec![String::from("faith"), String::from("schism")],
            HistoricalEventPayloadV1::FaithSchism {
                parent_id: parent.id,
                child_id,
            },
        );
        self.faiths.push(FaithRecordV1 {
            id: child_id,
            key: String::from("open-hand"),
            name,
            founded_year: self.year,
            origin_event: event,
            source_feature_id: parent.source_feature_id,
            parent_faith_id: Some(parent.id),
        });
        if let Some(population) = self.populations.iter_mut().find(|population| {
            population
                .lineage
                .iter()
                .filter(|share| share.parts_per_10_000 > 0)
                .count()
                > 1
        }) {
            population.faiths = add_affiliation(population.faiths.clone(), child_id, 3_500);
        }
    }

    fn finish(&mut self) {
        if self.history_completed {
            return;
        }
        self.push_event(
            HistoricalEventKindV1::HistoryCompleted,
            Vec::new(),
            None,
            self.last_event().into_iter().collect(),
            vec![String::from("history")],
            HistoricalEventPayloadV1::HistoryCompleted {
                elapsed_years: self.year,
            },
        );
        self.history_completed = true;
    }

    fn report(&self, title: String, seed: u64, years: u32) -> HistoricalReport {
        let mut populations = self.populations.clone();
        populations.sort_unstable_by_key(|population| population.id);
        let mut settlements = self.settlements.clone();
        settlements.sort_unstable_by_key(|settlement| settlement.location_id);
        let important_places = self.important_places();
        let starting_region = self.starting_region(&important_places);
        let lore = self.lore();
        let total_population = populations
            .iter()
            .map(|population| u64::from(population.people))
            .sum();
        let summary = HistorySummaryV1 {
            schema_version: HISTORY_SCHEMA_V1,
            history_id: self.config.id.clone(),
            seed,
            elapsed_years: self.year,
            total_population,
            population_cohorts: populations.len(),
            settlements: settlements.len(),
            cultures: self.cultures.len(),
            faiths: self.faiths.len(),
            institutions: self.institutions.len(),
            mixed_lineage_populations: populations
                .iter()
                .filter(|population| {
                    population
                        .lineage
                        .iter()
                        .filter(|share| share.parts_per_10_000 > 0)
                        .count()
                        > 1
                })
                .count(),
            first_contact_year: self.first_contact_year,
            event_count: self.events.len(),
        };
        let chronicle = render_chronicle(&title, seed, years, &summary, &important_places, &lore);
        HistoricalReport {
            title,
            seed,
            years,
            events: self.events.clone(),
            populations,
            settlements,
            cultures: self.cultures.clone(),
            faiths: self.faiths.clone(),
            institutions: self.institutions.clone(),
            polities: self.polities.clone(),
            lore,
            important_places,
            starting_region,
            summary,
            chronicle,
            open_route_ids: self.open_routes.iter().copied().collect(),
        }
    }

    fn important_places(&self) -> Vec<ImportantPlaceV1> {
        let population_by_location: BTreeMap<_, u32> = self
            .settlements
            .iter()
            .map(|settlement| (settlement.location_id, settlement.population))
            .collect();
        let mut important = self
            .graph
            .locations
            .iter()
            .map(|location| {
                let event_ids: Vec<_> = self
                    .events
                    .iter()
                    .filter(|event| event.location == Some(location.id))
                    .map(|event| event.id)
                    .collect();
                let population = population_by_location
                    .get(&location.id)
                    .copied()
                    .unwrap_or(0);
                let mixed = self.populations.iter().any(|cohort| {
                    cohort.location_id == location.id
                        && cohort
                            .lineage
                            .iter()
                            .filter(|share| share.parts_per_10_000 > 0)
                            .count()
                            > 1
                });
                let mut reasons = Vec::new();
                if population > 0 {
                    reasons.push(format!("{population} people at the close of history"));
                }
                if !location.feature_ids.is_empty() {
                    reasons.push(String::from("contains an unexplained prehuman trace"));
                }
                if mixed {
                    reasons.push(String::from("became a mixed-lineage contact settlement"));
                }
                if self.first_contact_location == Some(location.id) {
                    reasons.push(String::from("site of first cross-homeland contact"));
                }
                ImportantPlaceV1 {
                    location_id: location.id,
                    score: population
                        .saturating_add(event_ids.len() as u32 * 250)
                        .saturating_add(location.feature_ids.len() as u32 * 1_000)
                        .saturating_add(u32::from(mixed) * 5_000),
                    reasons,
                    event_ids,
                }
            })
            .collect::<Vec<_>>();
        important.sort_unstable_by_key(|place| (Reverse(place.score), place.location_id));
        important
    }

    fn starting_region(&self, important: &[ImportantPlaceV1]) -> StartingRegionV1 {
        let anchor = self
            .first_contact_location
            .or_else(|| important.first().map(|place| place.location_id))
            .unwrap_or(LocationId(1));
        let settled: BTreeSet<_> = self
            .settlements
            .iter()
            .filter(|settlement| settlement.abandoned_year.is_none())
            .map(|settlement| settlement.location_id)
            .collect();
        let mut queue = VecDeque::from([anchor]);
        let mut visited = BTreeSet::new();
        let mut settlement_ids = Vec::new();
        while let Some(location) = queue.pop_front() {
            if !visited.insert(location) {
                continue;
            }
            if settled.contains(&location) {
                settlement_ids.push(location);
                if settlement_ids.len() == 5 {
                    break;
                }
            }
            for route in &self.graph.routes {
                if self.open_routes.contains(&route.id)
                    && let Some(next) = other_endpoint(route.endpoints, location)
                {
                    queue.push_back(next);
                }
            }
        }
        if settlement_ids.len() < 5 {
            for settlement in &self.settlements {
                if !settlement_ids.contains(&settlement.location_id) {
                    settlement_ids.push(settlement.location_id);
                    if settlement_ids.len() == 5 {
                        break;
                    }
                }
            }
        }
        let population_ids = self
            .populations
            .iter()
            .filter(|population| settlement_ids.contains(&population.location_id))
            .map(|population| population.id)
            .collect::<Vec<_>>();
        let event_ids = self
            .events
            .iter()
            .filter(|event| {
                event
                    .location
                    .is_some_and(|location| settlement_ids.contains(&location))
            })
            .map(|event| event.id)
            .collect::<Vec<_>>();
        StartingRegionV1 {
            anchor_location_id: anchor,
            settlement_ids,
            population_ids,
            event_ids,
            summary: if let Some(year) = self.first_contact_year {
                format!(
                    "A five-settlement contact region shaped by cross-homeland first contact in Year {year}."
                )
            } else {
                String::from(
                    "A five-settlement region selected for population, routes, and surviving history.",
                )
            },
        }
    }

    fn lore(&self) -> Vec<LoreClaimV1> {
        let Some(contact) = self.first_contact_event else {
            return Vec::new();
        };
        self.config
            .contact_lore
            .iter()
            .enumerate()
            .filter_map(|(index, seed)| {
                let culture = self
                    .cultures
                    .iter()
                    .find(|culture| culture.key == seed.source_culture_key)?;
                let faith = seed.source_faith_key.as_ref().and_then(|key| {
                    self.faiths
                        .iter()
                        .find(|faith| faith.key == *key)
                        .map(|faith| faith.id)
                });
                Some(LoreClaimV1 {
                    id: u64::try_from(index).unwrap_or(u64::MAX).saturating_add(1),
                    title: seed.title.clone(),
                    text: seed.text.clone(),
                    source_culture_id: culture.id,
                    source_faith_id: faith,
                    about_events: vec![contact],
                    confidence_per_10_000: seed.confidence_per_10_000,
                })
            })
            .collect()
    }

    fn refresh_settlement_populations(&mut self) {
        let totals = self
            .populations
            .iter()
            .fold(BTreeMap::new(), |mut totals, population| {
                let entry = totals.entry(population.location_id).or_insert(0_u32);
                *entry = entry.saturating_add(population.people);
                totals
            });
        for settlement in &mut self.settlements {
            settlement.population = totals.get(&settlement.location_id).copied().unwrap_or(0);
        }
    }

    fn location(&self, id: LocationId) -> &merra_core::LocationRecordV1 {
        self.graph
            .locations
            .iter()
            .find(|location| location.id == id)
            .unwrap_or(&self.graph.locations[0])
    }

    fn push_event(
        &mut self,
        kind: HistoricalEventKindV1,
        subjects: Vec<HistoricalSubjectV1>,
        location: Option<LocationId>,
        causes: Vec<EventId>,
        tags: Vec<String>,
        payload: HistoricalEventPayloadV1,
    ) -> EventId {
        let id = EventId(self.next_event);
        self.next_event = self.next_event.saturating_add(1);
        self.events.push(HistoricalEventV1 {
            id,
            time: SimTime::from_day(
                u64::from(self.year).saturating_mul(u64::from(self.config.days_per_year)),
            ),
            kind,
            subjects,
            location,
            causes,
            tags,
            payload,
        });
        id
    }

    fn last_event(&self) -> Option<EventId> {
        self.events.last().map(|event| event.id)
    }
}

fn weighted_physiology(
    shares: &[AffiliationShareV1<LineageId>],
    lineages: &BTreeMap<LineageId, LineageDefinitionV1>,
    value: impl Fn(&LineageDefinitionV1) -> u16,
) -> u16 {
    let total = shares
        .iter()
        .filter_map(|share| {
            lineages.get(&share.id).map(|lineage| {
                u64::from(share.parts_per_10_000).saturating_mul(u64::from(value(lineage)))
            })
        })
        .sum::<u64>()
        / 10_000;
    u16::try_from(total).unwrap_or(u16::MAX)
}

fn lineage_share(population: &PopulationRecordV1, lineage_id: LineageId) -> u16 {
    population
        .lineage
        .iter()
        .find(|share| share.id == lineage_id)
        .map_or(0, |share| share.parts_per_10_000)
}

fn combine_affiliations<T: Copy + Ord>(
    first: &[AffiliationShareV1<T>],
    first_people: u32,
    second: &[AffiliationShareV1<T>],
    second_people: u32,
) -> Vec<AffiliationShareV1<T>> {
    let mut weights = BTreeMap::<T, u64>::new();
    for share in first {
        let entry = weights.entry(share.id).or_default();
        *entry = entry.saturating_add(
            u64::from(share.parts_per_10_000).saturating_mul(u64::from(first_people)),
        );
    }
    for share in second {
        let entry = weights.entry(share.id).or_default();
        *entry = entry.saturating_add(
            u64::from(share.parts_per_10_000).saturating_mul(u64::from(second_people)),
        );
    }
    normalize_weights(weights)
}

fn add_affiliation<T: Copy + Ord>(
    existing: Vec<AffiliationShareV1<T>>,
    id: T,
    requested_share: u16,
) -> Vec<AffiliationShareV1<T>> {
    let retained = 10_000_u64.saturating_sub(u64::from(requested_share));
    let mut weights = BTreeMap::new();
    for share in existing {
        weights.insert(
            share.id,
            u64::from(share.parts_per_10_000).saturating_mul(retained),
        );
    }
    let entry = weights.entry(id).or_default();
    *entry = entry.saturating_add(u64::from(requested_share).saturating_mul(10_000));
    normalize_weights(weights)
}

fn normalize_weights<T: Copy + Ord>(weights: BTreeMap<T, u64>) -> Vec<AffiliationShareV1<T>> {
    let total = weights.values().copied().sum::<u64>().max(1);
    let mut result = weights
        .into_iter()
        .filter(|(_, weight)| *weight > 0)
        .map(|(id, weight)| AffiliationShareV1 {
            id,
            parts_per_10_000: u16::try_from(weight.saturating_mul(10_000) / total)
                .unwrap_or(10_000),
        })
        .collect::<Vec<_>>();
    let assigned = result
        .iter()
        .map(|share| u32::from(share.parts_per_10_000))
        .sum::<u32>();
    if let Some(last) = result.last_mut() {
        last.parts_per_10_000 = last
            .parts_per_10_000
            .saturating_add(u16::try_from(10_000_u32.saturating_sub(assigned)).unwrap_or(0));
    }
    result
}

fn other_endpoint(endpoints: [LocationId; 2], source: LocationId) -> Option<LocationId> {
    if endpoints[0] == source {
        Some(endpoints[1])
    } else if endpoints[1] == source {
        Some(endpoints[0])
    } else {
        None
    }
}

fn select_founder_locations(
    founders: &[FounderSeedV1],
    graph: &PlaceGraphV1,
) -> Result<Vec<LocationId>, HistorySimulationError> {
    let mut selected = Vec::with_capacity(founders.len());
    for (index, founder) in founders.iter().enumerate() {
        let mut candidates = graph
            .locations
            .iter()
            .filter(|location| {
                !selected.contains(&location.id)
                    && location.tags.iter().any(|tag| tag == &founder.homeland_tag)
            })
            .collect::<Vec<_>>();
        candidates
            .sort_unstable_by_key(|location| (Reverse(location.carrying_capacity), location.id));
        let previous_for_tag = founders[..index]
            .iter()
            .zip(&selected)
            .filter(|(previous, _)| previous.homeland_tag == founder.homeland_tag)
            .map(|(_, location)| *location)
            .collect::<Vec<_>>();
        let location = match previous_for_tag.as_slice() {
            [] => candidates.first().map(|candidate| candidate.id),
            [source] => farthest_location(*source, &candidates, graph),
            sources => farthest_from_set(sources, &candidates, graph),
        }
        .or_else(|| candidates.first().map(|candidate| candidate.id))
        .ok_or(HistoryError::InsufficientSeedLocations)?;
        selected.push(location);
    }
    Ok(selected)
}

fn farthest_location(
    source: LocationId,
    locations: &[&merra_core::LocationRecordV1],
    graph: &PlaceGraphV1,
) -> Option<LocationId> {
    let source_region = graph
        .locations
        .iter()
        .find(|location| location.id == source)?
        .region?;
    locations
        .iter()
        .filter_map(|location| {
            let region = location.region?;
            Some((source_region.0.abs_diff(region.0), location.id))
        })
        .max()
        .map(|(_, id)| id)
}

fn farthest_from_set(
    sources: &[LocationId],
    locations: &[&merra_core::LocationRecordV1],
    graph: &PlaceGraphV1,
) -> Option<LocationId> {
    locations
        .iter()
        .filter(|location| !sources.contains(&location.id))
        .map(|location| {
            let minimum = sources
                .iter()
                .filter_map(|source| {
                    let first = graph
                        .locations
                        .iter()
                        .find(|candidate| candidate.id == *source)?;
                    Some(
                        first
                            .region?
                            .0
                            .abs_diff(location.region.unwrap_or(first.region?).0),
                    )
                })
                .min()
                .unwrap_or(0);
            (minimum, location.id)
        })
        .max()
        .map(|(_, id)| id)
}

fn history_roll(seed: u64, year: u32, domain: u32) -> u32 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"merra-history-roll-v1\0");
    hasher.update(&seed.to_le_bytes());
    hasher.update(&year.to_le_bytes());
    hasher.update(&domain.to_le_bytes());
    u32::from_le_bytes(
        hasher.finalize().as_bytes()[0..4]
            .try_into()
            .unwrap_or_default(),
    )
}

fn render_chronicle(
    title: &str,
    seed: u64,
    years: u32,
    summary: &HistorySummaryV1,
    important: &[ImportantPlaceV1],
    lore: &[LoreClaimV1],
) -> String {
    let contact = summary.first_contact_year.map_or_else(
        || String::from("No cross-homeland contact occurred."),
        |year| format!("Separated histories first touched in Year {year}."),
    );
    let places = important
        .iter()
        .take(3)
        .map(|place| {
            format!(
                "- Location #{}: score {}\n",
                place.location_id.0, place.score
            )
        })
        .collect::<String>();
    let claims = lore
        .iter()
        .map(|claim| format!("- **{}:** {}\n", claim.title, claim.text))
        .collect::<String>();
    format!(
        "# Chronicle: {title}\n\n\
         - Seed: `{seed}`\n\
         - Historical span: {years} years\n\
         - Population: {}\n\
         - Settlements: {}\n\
         - Cultures: {}\n\
         - Faiths: {}\n\
         - Mixed-lineage populations: {}\n\n\
         ## The Crossing\n\n{contact}\n\n\
         ## Important Places\n\n{places}\n\
         ## What Later Sources Claimed\n\n{claims}",
        summary.total_population,
        summary.settlements,
        summary.cultures,
        summary.faiths,
        summary.mixed_lineage_populations,
    )
}

#[cfg(test)]
mod tests {
    use merra_core::{
        CultureSeedV1, FaithSeedV1, FeatureId, FeatureKindV1, FounderSeedV1, HistoryConfigV1,
        LineageDefinitionV1, LineageId, LineagePhysiologyV1, LocationId, LocationRecordV1,
        LoreSeedV1, MythicMotifConfigV1, PlaceAffordanceV1, PlaceGraphV1, RouteId, RouteKindV1,
        RouteRecordV1, WORLD_GENESIS_SCHEMA_V1, WorldFeatureV1, WorldGenesisConfigV1,
    };
    use merra_worldgen::generate_world;

    use super::{run_history, run_history_on_graph};

    fn history_config(years: u32) -> HistoryConfigV1 {
        HistoryConfigV1 {
            schema_version: 1,
            id: String::from("history-test"),
            title: String::from("The First Histories"),
            days_per_year: 360,
            years,
            initial_population_per_cohort: 500,
            lineages: vec![
                LineageDefinitionV1 {
                    id: LineageId(1),
                    key: String::from("human"),
                    name: String::from("Human"),
                    physiology: LineagePhysiologyV1 {
                        adult_mortality_multiplier_per_10_000: 10_000,
                        physical_power_per_10_000: 10_000,
                        movement_speed_per_10_000: 10_000,
                        sustenance_demand_per_10_000: 10_000,
                    },
                },
                LineageDefinitionV1 {
                    id: LineageId(2),
                    key: String::from("orc"),
                    name: String::from("Orc"),
                    physiology: LineagePhysiologyV1 {
                        adult_mortality_multiplier_per_10_000: 7_500,
                        physical_power_per_10_000: 12_500,
                        movement_speed_per_10_000: 10_000,
                        sustenance_demand_per_10_000: 11_250,
                    },
                },
            ],
            founders: (1..=4)
                .map(|number| {
                    let isolated = number == 4;
                    FounderSeedV1 {
                        lineage_id: if isolated { LineageId(2) } else { LineageId(1) },
                        homeland_tag: if isolated {
                            String::from("isolated_homeland")
                        } else {
                            String::from("primary_homeland")
                        },
                        culture: CultureSeedV1 {
                            key: format!("tradition-{number}"),
                            name: if isolated {
                                String::from("Keepers of the Ring")
                            } else {
                                format!("Human Tradition {number}")
                            },
                            ritual_days_per_year: if isolated { 24 } else { 6 },
                            sacred_contribution_per_10_000: if isolated { 750 } else { 200 },
                            institutional_preservation_per_10_000: if isolated {
                                14_000
                            } else {
                                10_000
                            },
                            faith_transmission_per_10_000: if isolated { 12_500 } else { 10_000 },
                        },
                    }
                })
                .collect(),
            faiths: vec![
                FaithSeedV1 {
                    key: String::from("ring-witness"),
                    name: String::from("The Ring Witness"),
                    culture_key: String::from("tradition-4"),
                    founded_year: 0,
                    source_motif_id: Some(String::from("stone-rings")),
                    tags: vec![String::from("mythic-trace")],
                    founding_institution: true,
                },
                FaithSeedV1 {
                    key: String::from("river-witness"),
                    name: String::from("The River Witness"),
                    culture_key: String::from("tradition-1"),
                    founded_year: 120,
                    source_motif_id: None,
                    tags: vec![String::from("river")],
                    founding_institution: false,
                },
            ],
            contact_culture: CultureSeedV1 {
                key: String::from("contact"),
                name: String::from("Tidebound"),
                ritual_days_per_year: 12,
                sacred_contribution_per_10_000: 400,
                institutional_preservation_per_10_000: 11_500,
                faith_transmission_per_10_000: 11_000,
            },
            contact_lore: vec![
                LoreSeedV1 {
                    title: String::from("The Discovery"),
                    text: String::from("The first homeland claimed discovery."),
                    source_culture_key: String::from("tradition-1"),
                    source_faith_key: None,
                    confidence_per_10_000: 7_200,
                },
                LoreSeedV1 {
                    title: String::from("The Return"),
                    text: String::from("The isolated homeland claimed return."),
                    source_culture_key: String::from("tradition-4"),
                    source_faith_key: Some(String::from("ring-witness")),
                    confidence_per_10_000: 8_400,
                },
            ],
            contact_navigation_threshold: 470,
        }
    }

    fn world() -> Result<merra_core::SurfaceWorldV1, Box<dyn std::error::Error>> {
        Ok(generate_world(
            &WorldGenesisConfigV1 {
                schema_version: WORLD_GENESIS_SCHEMA_V1,
                id: String::from("history-world"),
                title: String::from("Before Memory"),
                width: 64,
                height: 48,
                plate_count: 8,
                land_fraction_per_10_000: 4_800,
                island_land_fraction_per_10_000: 800,
                island_separation: 8,
                place_count: 20,
                mythic_motifs: vec![MythicMotifConfigV1 {
                    id: String::from("stone-rings"),
                    name: String::from("Stone Ring"),
                    count: 4,
                }],
            },
            42,
        )?)
    }

    #[test]
    fn history_is_deterministic_and_preserves_normalized_mixtures()
    -> Result<(), Box<dyn std::error::Error>> {
        let world = world()?;
        let first = run_history(&world, history_config(600), 42)?;
        let second = run_history(&world, history_config(600), 42)?;
        assert_eq!(first, second);
        assert_eq!(first.summary.elapsed_years, 600);
        assert!(first.summary.settlements >= 5);
        assert!(first.summary.first_contact_year.is_some());
        assert!(first.summary.mixed_lineage_populations > 0);
        assert_eq!(first.starting_region.settlement_ids.len(), 5);
        assert!(first.populations.iter().all(|population| {
            population
                .lineage
                .iter()
                .map(|share| u32::from(share.parts_per_10_000))
                .sum::<u32>()
                == 10_000
        }));
        assert!(
            first
                .events
                .iter()
                .all(|event| event.causes.iter().all(|cause| cause.0 < event.id.0))
        );
        Ok(())
    }

    #[test]
    fn orc_physiology_is_data_not_a_separate_system() {
        let config = history_config(600);
        let human = &config.lineages[0].physiology;
        let orc = &config.lineages[1].physiology;
        assert!(
            orc.adult_mortality_multiplier_per_10_000 < human.adult_mortality_multiplier_per_10_000
        );
        assert!(orc.physical_power_per_10_000 > human.physical_power_per_10_000);
        assert_eq!(
            orc.movement_speed_per_10_000,
            human.movement_speed_per_10_000
        );
    }

    #[test]
    fn orbital_habitat_graph_uses_the_same_history_engine() -> Result<(), Box<dyn std::error::Error>>
    {
        let locations = (1..=9)
            .map(|raw_id| LocationRecordV1 {
                id: LocationId(raw_id),
                name: format!("Habitat {raw_id}"),
                region: None,
                tags: vec![if raw_id <= 6 {
                    String::from("primary_homeland")
                } else if raw_id <= 8 {
                    String::from("isolated_homeland")
                } else {
                    String::from("third_homeland")
                }],
                carrying_capacity: 8_000,
                hazard_per_10_000: 200,
                affordances: vec![
                    PlaceAffordanceV1 {
                        id: String::from("food"),
                        value_per_10_000: 7_500,
                    },
                    PlaceAffordanceV1 {
                        id: String::from("navigation"),
                        value_per_10_000: 8_000,
                    },
                ],
                feature_ids: if raw_id == 7 {
                    vec![FeatureId(1)]
                } else {
                    Vec::new()
                },
            })
            .collect();
        let mut routes = (1..=5)
            .map(|raw_id| RouteRecordV1 {
                id: RouteId(raw_id),
                endpoints: [LocationId(raw_id), LocationId(raw_id + 1)],
                kind: RouteKindV1::Abstract,
                travel_cost: 10,
                capacity: 2_000,
                locked: false,
                required_capability: None,
            })
            .collect::<Vec<_>>();
        routes.push(RouteRecordV1 {
            id: RouteId(6),
            endpoints: [LocationId(7), LocationId(8)],
            kind: RouteKindV1::Abstract,
            travel_cost: 10,
            capacity: 2_000,
            locked: false,
            required_capability: None,
        });
        routes.push(RouteRecordV1 {
            id: RouteId(7),
            endpoints: [LocationId(6), LocationId(7)],
            kind: RouteKindV1::Abstract,
            travel_cost: 80,
            capacity: 500,
            locked: true,
            required_capability: Some(String::from("navigation")),
        });
        routes.push(RouteRecordV1 {
            id: RouteId(8),
            endpoints: [LocationId(8), LocationId(9)],
            kind: RouteKindV1::Abstract,
            travel_cost: 20,
            capacity: 1_000,
            locked: false,
            required_capability: None,
        });
        let feature = WorldFeatureV1 {
            id: FeatureId(1),
            name: String::from("The Silent Array"),
            kind: FeatureKindV1::MythicTrace {
                motif_id: String::from("stone-rings"),
            },
            regions: Vec::new(),
            description: String::from("An unexplained pre-colony signal array."),
        };
        let mut config = history_config(300);
        config.contact_navigation_threshold = 260;
        config.lineages.push(LineageDefinitionV1 {
            id: LineageId(3),
            key: String::from("synthetic"),
            name: String::from("Synthetic"),
            physiology: LineagePhysiologyV1 {
                adult_mortality_multiplier_per_10_000: 9_000,
                physical_power_per_10_000: 8_000,
                movement_speed_per_10_000: 11_000,
                sustenance_demand_per_10_000: 6_000,
            },
        });
        config.founders.push(FounderSeedV1 {
            lineage_id: LineageId(3),
            homeland_tag: String::from("third_homeland"),
            culture: CultureSeedV1 {
                key: String::from("relay-born"),
                name: String::from("Relay-Born"),
                ritual_days_per_year: 0,
                sacred_contribution_per_10_000: 0,
                institutional_preservation_per_10_000: 11_000,
                faith_transmission_per_10_000: 9_000,
            },
        });
        let report =
            run_history_on_graph(PlaceGraphV1 { locations, routes }, &[feature], config, 42)?;
        assert_eq!(report.summary.elapsed_years, 300);
        assert!(report.summary.first_contact_year.is_some());
        assert!(report.starting_region.settlement_ids.len() >= 5);
        assert!(report.populations.iter().any(|population| {
            population
                .lineage
                .iter()
                .any(|share| share.id == LineageId(3))
        }));
        Ok(())
    }
}
