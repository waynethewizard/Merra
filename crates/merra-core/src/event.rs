//! Structured historical events.

use serde::{Deserialize, Serialize};

use crate::{EventId, LocationId, PersonId, SimTime};

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
