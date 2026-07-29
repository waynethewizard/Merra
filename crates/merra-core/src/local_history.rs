//! Contracts for projecting aggregate history into five detailed settlements.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    CultureId, CultureRecordV1, EventId, EventPayloadV1, FaithId, FaithRecordV1, HistoricalEventV1,
    HouseholdId, HouseholdRecordV1, InstitutionId, InstitutionRecordV1, LocationId, LoreClaimV1,
    PersonId, PersonRecordV1, PopulationId, PopulationRecordV1, RouteId, ScenarioError, ScenarioV1,
    SettlementRecordV1, SimulationSummaryV1, SourceVersionV1, StartingRegionV1, WorldEventV1,
};

/// Current local-history schema.
pub const LOCAL_HISTORY_SCHEMA_V1: u32 = 1;

/// Current person-level local-history playback schema.
pub const LOCAL_PLAYBACK_SCHEMA_V1: u32 = 1;

/// Rules for one deterministic projection into detailed local simulation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalHistoryConfigV1 {
    /// Schema version, currently `1`.
    pub schema_version: u32,
    /// Stable machine-readable identifier.
    pub id: String,
    /// Human-readable local-age title.
    pub title: String,
    /// Number of detailed years to simulate after the macro-history handoff.
    pub years: u32,
    /// Number of selected settlements required from the historical handoff.
    pub settlement_count: u16,
    /// Calendar days represented by one unit of graph travel cost.
    pub travel_days_per_cost: u16,
    /// Existing person, mortality, and family rules used at detailed resolution.
    pub detailed_scenario: ScenarioV1,
}

impl LocalHistoryConfigV1 {
    /// Validates local projection rules before a simulation begins.
    pub fn validate(&self) -> Result<(), LocalHistoryConfigError> {
        if self.schema_version != LOCAL_HISTORY_SCHEMA_V1 {
            return Err(LocalHistoryConfigError::UnsupportedSchema(
                self.schema_version,
            ));
        }
        if self.id.trim().is_empty() || self.title.trim().is_empty() {
            return Err(LocalHistoryConfigError::MissingIdentity);
        }
        if self.years == 0 || self.travel_days_per_cost == 0 {
            return Err(LocalHistoryConfigError::EmptyDurationOrTravelScale);
        }
        if self.settlement_count != 5 {
            return Err(LocalHistoryConfigError::NotFiveSettlements(
                self.settlement_count,
            ));
        }
        self.detailed_scenario.validate()?;
        if !self.detailed_scenario.family.enabled {
            return Err(LocalHistoryConfigError::FamiliesRequired);
        }
        if self.detailed_scenario.population.initial_people < u32::from(self.settlement_count) {
            return Err(LocalHistoryConfigError::InsufficientDetailedPeople);
        }
        Ok(())
    }
}

/// Historical evidence needed at the macro-to-local boundary.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RegionalHistoryV1 {
    /// Title of the completed aggregate history.
    pub history_title: String,
    /// Final macro-history year at which projection begins.
    pub projection_year: u32,
    /// The selected five-settlement handoff.
    pub starting_region: StartingRegionV1,
    /// Aggregate populations present in the selected settlements.
    pub populations: Vec<PopulationRecordV1>,
    /// Settlement records for the selected locations.
    pub settlements: Vec<SettlementRecordV1>,
    /// Cultures referenced by selected aggregate populations and claims.
    pub cultures: Vec<CultureRecordV1>,
    /// Faiths referenced by selected aggregate populations and claims.
    pub faiths: Vec<FaithRecordV1>,
    /// Institutions physically or culturally connected to selected populations.
    pub institutions: Vec<InstitutionRecordV1>,
    /// Claims that refer to events retained by the starting region.
    pub lore: Vec<LoreClaimV1>,
    /// Relevant macro events retained as causal context.
    pub events: Vec<HistoricalEventV1>,
    /// Routes available when detailed simulation begins.
    pub open_route_ids: Vec<RouteId>,
}

/// Exact share of one aggregate population represented by a sampled household.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PopulationAllocationV1 {
    /// Aggregate cohort being represented.
    pub population_id: PopulationId,
    /// Number of macro people assigned to the detailed household as evidence.
    pub people: u32,
}

/// Historical knowledge and affiliation inherited by one detailed household.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HouseholdHistoricalContextV1 {
    /// Stable household identity from the detailed simulation.
    pub household_id: HouseholdId,
    /// The household's one authoritative residence.
    pub residence_id: LocationId,
    /// Exact macro population allocations at the projection boundary.
    pub represented_populations: Vec<PopulationAllocationV1>,
    /// Cultures inherited from represented populations or founding households.
    pub culture_ids: Vec<CultureId>,
    /// Faiths inherited from represented populations or founding households.
    pub faith_ids: Vec<FaithId>,
    /// Local historical institutions visible to this household.
    pub institution_ids: Vec<InstitutionId>,
    /// Historically situated claims available through the household's cultures.
    pub lore_claim_ids: Vec<u64>,
}

/// Why a household selected its residence.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResidenceReasonV1 {
    /// Initial households were allocated to reconcile aggregate populations.
    MacroProjection,
    /// The location had the greatest count of living close kin.
    LivingKin,
    /// No location had more kin, so total road cost decided the destination.
    ShortestJourney,
    /// Kin and road cost tied, so an isolated seeded rank decided.
    SeededTieBreak,
}

/// Explainable residence choice made when a household forms.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ResidenceDecisionV1 {
    /// Household receiving a residence.
    pub household_id: HouseholdId,
    /// Absolute detailed-simulation day of the decision.
    pub settled_day: u64,
    /// Previous household residences of the founding members.
    pub origin_location_ids: Vec<LocationId>,
    /// Selected household residence.
    pub destination_location_id: LocationId,
    /// People who traveled to form the household.
    pub traveler_ids: Vec<PersonId>,
    /// Shortest-path route identities used by the travelers.
    pub route_ids: Vec<RouteId>,
    /// Greatest shortest-path travel cost paid by a traveler.
    pub travel_cost: u32,
    /// Calendar travel time derived from cost and the configured scale.
    pub travel_days: u32,
    /// Living close kin counted at the selected destination.
    pub living_kin_support: u16,
    /// Dominant explanation for the deterministic choice.
    pub reason: ResidenceReasonV1,
    /// Earlier local events that establish the household and its origins.
    pub causes: Vec<EventId>,
}

/// Shortest available connection between two selected settlements.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalConnectionV1 {
    /// First selected settlement, ordered by stable identity.
    pub from: LocationId,
    /// Second selected settlement.
    pub to: LocationId,
    /// Sum of route costs along the deterministic shortest path.
    pub travel_cost: u32,
    /// Calendar travel time derived from cost.
    pub travel_days: u32,
    /// Stable route sequence from origin to destination.
    pub route_ids: Vec<RouteId>,
    /// Location sequence including both endpoints and any intermediate places.
    pub path: Vec<LocationId>,
}

/// One selected settlement with macro provenance and detailed vital statistics.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalSettlementRecordV1 {
    /// Stable location identity inherited from world generation.
    pub location_id: LocationId,
    /// Historical place name.
    pub name: String,
    /// Aggregate population at the projection boundary.
    pub macro_population: u32,
    /// Macro population exactly allocated across initial sample households.
    pub represented_population: u32,
    /// Detailed people initially sampled at this settlement.
    pub initial_sample_people: u32,
    /// Detailed people alive here at the end of the local run.
    pub final_living_people: u32,
    /// Detailed births recorded at this location.
    pub births: u32,
    /// Detailed deaths recorded at this location.
    pub deaths: u32,
    /// People arriving in newly formed households.
    pub arrivals: u32,
    /// People leaving prior households at this location.
    pub departures: u32,
    /// Active detailed households at the end of the run.
    pub active_households: u32,
    /// Historical institutions physically present here.
    pub institution_ids: Vec<InstitutionId>,
    /// Macro events retained for this location.
    pub historical_event_ids: Vec<EventId>,
}

/// Compact evidence for one five-village run.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalHistorySummaryV1 {
    /// Output schema version.
    pub schema_version: u32,
    /// Local-history scenario identifier.
    pub local_history_id: String,
    /// Root deterministic seed.
    pub seed: u64,
    /// Macro-history year at projection.
    pub projection_year: u32,
    /// Detailed years elapsed.
    pub elapsed_years: u32,
    /// Number of selected settlements.
    pub settlements: usize,
    /// Aggregate people reconciled at the handoff.
    pub macro_population: u64,
    /// Aggregate people allocated across sampled initial households.
    pub represented_population: u64,
    /// Detailed people alive at the end of the run.
    pub living_sample_people: u32,
    /// Detailed births by place.
    pub births: u32,
    /// Detailed deaths by place.
    pub deaths: u32,
    /// Household residence choices after the initial projection.
    pub residence_decisions: u32,
    /// New households whose members crossed at least one settlement boundary.
    pub household_migrations: u32,
    /// Number of local events with an authoritative location.
    pub located_events: usize,
}

/// Stable person metadata used by the local-history playback.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalPlaybackPersonV1 {
    /// Stable detailed person identity.
    pub id: PersonId,
    /// Human-readable name at the end of the run.
    pub name: String,
    /// Founder generation is zero.
    pub generation: u16,
    /// Complete age at the projection boundary for initial people.
    pub starting_age_years: u16,
    /// Absolute birth day for people born during local history.
    pub birth_day: Option<u64>,
    /// Absolute death day when the person died during local history.
    pub death_day: Option<u64>,
    /// Stable parent identities, empty for the projected founders.
    pub parent_ids: Vec<PersonId>,
}

/// One location-changing or vital event required to replay sampled people.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LocalPlaybackEventV1 {
    /// A household and its founders acquired one authoritative residence.
    HouseholdSettled {
        /// Source event identity in the complete local event stream.
        event_id: EventId,
        /// Absolute local-simulation day.
        day: u64,
        /// Household selecting a residence.
        household_id: HouseholdId,
        /// Prior residences of the founding members.
        origin_location_ids: Vec<LocationId>,
        /// Selected residence.
        destination_location_id: LocationId,
        /// People who moved with the household.
        traveler_ids: Vec<PersonId>,
        /// Shortest-path route identities used by the travelers.
        route_ids: Vec<RouteId>,
        /// Greatest shortest-path cost paid by a traveler.
        travel_cost: u32,
        /// Calendar travel time implied by the configured scale.
        travel_days: u32,
        /// Living close kin counted at the destination.
        living_kin_support: u16,
        /// Dominant deterministic residence rule.
        reason: ResidenceReasonV1,
    },
    /// A sampled person was born at one authoritative location.
    PersonBorn {
        /// Source event identity in the complete local event stream.
        event_id: EventId,
        /// Absolute local-simulation day.
        day: u64,
        /// Stable newborn identity.
        person_id: PersonId,
        /// Household into which the person was born.
        household_id: HouseholdId,
        /// Authoritative birthplace.
        location_id: LocationId,
    },
    /// A sampled person died at one authoritative location.
    PersonDied {
        /// Source event identity in the complete local event stream.
        event_id: EventId,
        /// Absolute local-simulation day.
        day: u64,
        /// Stable identity of the person who died.
        person_id: PersonId,
        /// Complete age at death.
        age_years: u64,
        /// Authoritative place of death.
        location_id: LocationId,
    },
}

/// Compact, event-faithful stream used to animate sampled local lives.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalHistoryPlaybackV1 {
    /// Playback schema version.
    pub schema_version: u32,
    /// Root deterministic local-history seed.
    pub seed: u64,
    /// Macro-history year at which local playback begins.
    pub projection_year: u32,
    /// Number of detailed years in the playback.
    pub elapsed_years: u32,
    /// Calendar days represented by one local year.
    pub days_per_year: u16,
    /// Stable metadata for every sampled person who ever lived in the run.
    pub people: Vec<LocalPlaybackPersonV1>,
    /// Ordered placements, births, deaths, and later residence choices.
    pub events: Vec<LocalPlaybackEventV1>,
}

/// Reproducibility metadata for a local-history run.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalHistoryManifestV1 {
    /// Manifest schema version.
    pub schema_version: u32,
    /// Structured local-event schema version.
    pub event_schema_version: u32,
    /// Merra package version.
    pub merra_version: String,
    /// Selected Bevy release.
    pub bevy_version: String,
    /// Pinned Rust toolchain.
    pub rust_version: String,
    /// Source revision associated with the run.
    pub source: SourceVersionV1,
    /// Local-history scenario identifier.
    pub local_history_id: String,
    /// Hash of the exact local-history scenario bytes.
    pub local_history_hash: String,
    /// Hash of the source world bytes.
    pub world_hash: String,
    /// Hash of the regional-history handoff bytes.
    pub regional_history_hash: String,
    /// Root deterministic seed.
    pub seed: u64,
    /// Detailed years simulated.
    pub years: u32,
}

/// Complete inspectable five-settlement result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocalHistoryReportV1 {
    /// Human-readable title.
    pub title: String,
    /// Root deterministic seed.
    pub seed: u64,
    /// Detailed simulation result.
    pub simulation_summary: SimulationSummaryV1,
    /// Person records whose current place derives from household residence.
    pub people: Vec<PersonRecordV1>,
    /// Household records, each with exactly one residence in this report.
    pub households: Vec<HouseholdRecordV1>,
    /// Original and derived local events in stable causal order.
    pub events: Vec<WorldEventV1>,
    /// One exact historical context record per household.
    pub household_contexts: Vec<HouseholdHistoricalContextV1>,
    /// Explainable residence decisions in stable household order.
    pub residence_decisions: Vec<ResidenceDecisionV1>,
    /// Pairwise shortest road connections between the selected settlements.
    pub connections: Vec<LocalConnectionV1>,
    /// Selected settlements with macro and detailed measurements.
    pub settlements: Vec<LocalSettlementRecordV1>,
    /// Relevant claims inherited from aggregate history.
    pub lore: Vec<LoreClaimV1>,
    /// Cultures represented in the selected settlements.
    pub cultures: Vec<CultureRecordV1>,
    /// Faiths represented in the selected settlements.
    pub faiths: Vec<FaithRecordV1>,
    /// Historical institutions visible at local scale.
    pub institutions: Vec<InstitutionRecordV1>,
    /// Compact local-history measurements.
    pub summary: LocalHistorySummaryV1,
    /// Human-readable place history.
    pub chronicle: String,
}

impl LocalHistoryPlaybackV1 {
    /// Extracts the minimal person-level stream needed for a faithful animation.
    #[must_use]
    pub fn from_report(report: &LocalHistoryReportV1) -> Self {
        let people = report
            .people
            .iter()
            .map(|person| LocalPlaybackPersonV1 {
                id: person.id,
                name: person.name.clone(),
                generation: person.generation,
                starting_age_years: person.starting_age_years,
                birth_day: person.birth_day,
                death_day: person.death_day,
                parent_ids: person.parent_ids.clone(),
            })
            .collect();
        let events = report
            .events
            .iter()
            .filter_map(|event| match &event.payload {
                EventPayloadV1::HouseholdSettled {
                    household_id,
                    origin_location_ids,
                    destination_location_id,
                    traveler_ids,
                    route_ids,
                    travel_cost,
                    travel_days,
                    living_kin_support,
                    reason,
                } => Some(LocalPlaybackEventV1::HouseholdSettled {
                    event_id: event.id,
                    day: event.time.day(),
                    household_id: *household_id,
                    origin_location_ids: origin_location_ids.clone(),
                    destination_location_id: *destination_location_id,
                    traveler_ids: traveler_ids.clone(),
                    route_ids: route_ids.clone(),
                    travel_cost: *travel_cost,
                    travel_days: *travel_days,
                    living_kin_support: *living_kin_support,
                    reason: *reason,
                }),
                EventPayloadV1::PersonBorn {
                    person_id,
                    household_id,
                    ..
                } => event
                    .location
                    .map(|location_id| LocalPlaybackEventV1::PersonBorn {
                        event_id: event.id,
                        day: event.time.day(),
                        person_id: *person_id,
                        household_id: *household_id,
                        location_id,
                    }),
                EventPayloadV1::PersonDied {
                    person_id,
                    age_years,
                    ..
                } => event
                    .location
                    .map(|location_id| LocalPlaybackEventV1::PersonDied {
                        event_id: event.id,
                        day: event.time.day(),
                        person_id: *person_id,
                        age_years: *age_years,
                        location_id,
                    }),
                _ => None,
            })
            .collect();
        Self {
            schema_version: LOCAL_PLAYBACK_SCHEMA_V1,
            seed: report.seed,
            projection_year: report.summary.projection_year,
            elapsed_years: report.summary.elapsed_years,
            days_per_year: report.simulation_summary.days_per_year,
            people,
            events,
        }
    }
}

/// Invalid local-history configuration.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum LocalHistoryConfigError {
    /// Unknown schema version.
    #[error("unsupported local-history schema {0}")]
    UnsupportedSchema(u32),
    /// Missing stable or display identity.
    #[error("local-history id and title must not be blank")]
    MissingIdentity,
    /// Empty detailed duration or travel-time scale.
    #[error("local-history duration and travel scale must be positive")]
    EmptyDurationOrTravelScale,
    /// Cycle 5 requires an exact five-settlement handoff.
    #[error("local history requires exactly five settlements, found {0}")]
    NotFiveSettlements(u16),
    /// Detailed local history depends on households.
    #[error("local history requires enabled family and household rules")]
    FamiliesRequired,
    /// At least one sampled person is needed per settlement.
    #[error("detailed sample population is smaller than the settlement count")]
    InsufficientDetailedPeople,
    /// Nested detailed scenario is invalid.
    #[error(transparent)]
    Scenario(#[from] ScenarioError),
}
