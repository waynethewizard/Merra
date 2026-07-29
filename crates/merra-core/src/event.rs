//! Structured historical events.

use serde::{Deserialize, Serialize};

use crate::{EventId, HouseholdId, LocationId, PersonId, ResidenceReasonV1, RouteId, SimTime};

/// Stable event categories in schema version 1.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKindV1 {
    /// A simulation was initialized.
    SimulationStarted,
    /// The scenario's initial people entered the world.
    PopulationInitialized,
    /// The clock advanced.
    TimeAdvanced,
    /// A named season began.
    SeasonBegan,
    /// A household was established.
    HouseholdFormed,
    /// Two people formed a partnership.
    PartnershipFormed,
    /// A partnership ended after a death.
    PartnershipEnded,
    /// A person was born.
    PersonBorn,
    /// A household ceased to have living members.
    HouseholdDissolved,
    /// A newly formed household selected one residence and traveled there.
    HouseholdSettled,
    /// A person died.
    PersonDied,
    /// A requested run completed.
    SimulationCompleted,
}

/// Typed event details in schema version 1.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventPayloadV1 {
    /// Initial scenario and seed information.
    SimulationStarted {
        /// Scenario identifier.
        scenario_id: String,
        /// Root deterministic seed.
        seed: u64,
    },
    /// Initial population count.
    PopulationInitialized {
        /// Number of people spawned at the epoch.
        people: u32,
    },
    /// One explicit time advancement.
    TimeAdvanced {
        /// Starting absolute day.
        from_day: u64,
        /// Ending absolute day.
        to_day: u64,
        /// Number of elapsed days.
        elapsed_days: u64,
    },
    /// A named season began at an exact calendar boundary.
    SeasonBegan {
        /// Stable season identifier from the scenario.
        season_id: String,
        /// Human-readable season name.
        season_name: String,
        /// Zero-based year containing the season start.
        year: u64,
    },
    /// A new household acquired a stable identity.
    HouseholdFormed {
        /// Stable household identifier.
        household_id: HouseholdId,
        /// Human-readable household name.
        name: String,
        /// Surname inherited by children born into this household.
        surname: String,
        /// Founding members in stable person order.
        member_ids: Vec<PersonId>,
    },
    /// Two living adults formed a household partnership.
    PartnershipFormed {
        /// Stable household established by the partnership.
        household_id: HouseholdId,
        /// Partners in stable identity order.
        partners: [PersonId; 2],
    },
    /// A household partnership ended because one partner died.
    PartnershipEnded {
        /// Partners in stable identity order.
        partners: [PersonId; 2],
        /// Partner whose death ended the partnership.
        deceased_id: PersonId,
    },
    /// A child was born into a household.
    PersonBorn {
        /// Stable identity assigned to the child.
        person_id: PersonId,
        /// Human-readable name assigned at birth.
        name: String,
        /// Parents in stable identity order.
        parent_ids: [PersonId; 2],
        /// Household into which the child was born.
        household_id: HouseholdId,
        /// Founder generation is zero.
        generation: u16,
    },
    /// A household ceased to have living members.
    HouseholdDissolved {
        /// Stable household identity.
        household_id: HouseholdId,
        /// Human-readable household name at dissolution.
        name: String,
    },
    /// A household selected one residence using kin support and road cost.
    HouseholdSettled {
        /// Stable household identity.
        household_id: HouseholdId,
        /// Previous residences of its founding members.
        origin_location_ids: Vec<LocationId>,
        /// New authoritative household residence.
        destination_location_id: LocationId,
        /// People who traveled.
        traveler_ids: Vec<PersonId>,
        /// Stable shortest-path route identities used.
        route_ids: Vec<RouteId>,
        /// Greatest shortest-path cost paid by a traveler.
        travel_cost: u32,
        /// Calendar days implied by the configured travel scale.
        travel_days: u32,
        /// Living close kin counted at the destination.
        living_kin_support: u16,
        /// Dominant deterministic selection rule.
        reason: ResidenceReasonV1,
    },
    /// An explainable natural death.
    PersonDied {
        /// Stable identity of the person who died.
        person_id: PersonId,
        /// Human-readable name at the time of death.
        name: String,
        /// Complete age at death.
        age_years: u64,
        /// Integer mortality threshold used for the check.
        annual_deaths_per_10_000: u16,
    },
    /// Final run extent.
    SimulationCompleted {
        /// Final absolute day.
        final_day: u64,
        /// Number of complete elapsed scenario years.
        elapsed_years: u64,
    },
}

/// An omniscient world event from which later memories and records may derive.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorldEventV1 {
    /// Stable event identifier.
    pub id: EventId,
    /// Simulation time at which the event occurred.
    pub time: SimTime,
    /// Stable event category.
    pub kind: EventKindV1,
    /// People directly involved in the event.
    pub actors: Vec<PersonId>,
    /// Optional location of the event.
    pub location: Option<LocationId>,
    /// Earlier events that causally contributed to this event.
    pub causes: Vec<EventId>,
    /// Searchable, non-authoritative labels.
    pub tags: Vec<String>,
    /// Typed event-specific details.
    pub payload: EventPayloadV1,
}
