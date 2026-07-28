//! Versioned output types written by the headless runner.

use serde::{Deserialize, Serialize};

use crate::{HouseholdId, PersonId};

/// Source revision associated with a run.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceVersionV1 {
    /// Git commit when it can be determined.
    pub git_commit: Option<String>,
    /// Whether tracked files differed from the commit.
    pub dirty: Option<bool>,
}

/// Reproducibility metadata for a simulation run.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RunManifestV1 {
    /// Manifest schema version.
    pub schema_version: u32,
    /// Structured-event schema version.
    pub event_schema_version: u32,
    /// Scenario schema version.
    pub scenario_schema_version: u32,
    /// Merra package version.
    pub merra_version: String,
    /// Selected Bevy release.
    pub bevy_version: String,
    /// Pinned Rust toolchain.
    pub rust_version: String,
    /// Source revision information.
    pub source: SourceVersionV1,
    /// Stable scenario identifier.
    pub scenario_id: String,
    /// BLAKE3 hash of the exact scenario bytes.
    pub scenario_hash: String,
    /// Root deterministic seed.
    pub seed: u64,
    /// Requested number of years.
    pub years: u32,
    /// Resulting number of simulation days.
    pub days: u64,
}

/// Compact machine-readable result for a simulation run.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SimulationSummaryV1 {
    /// Output schema version.
    pub schema_version: u32,
    /// Scenario identifier.
    pub scenario_id: String,
    /// Root seed.
    pub seed: u64,
    /// Final absolute day.
    pub elapsed_days: u64,
    /// Complete elapsed scenario years.
    pub elapsed_years: u64,
    /// Calendar days in one scenario year.
    pub days_per_year: u16,
    /// Number of structured events emitted.
    pub event_count: usize,
    /// People present at the scenario epoch.
    pub initial_population: u32,
    /// People alive at the end of the run.
    pub living_population: u32,
    /// Deaths recorded during the run.
    pub deaths: u32,
}

/// Inspectable person state at the end of a run.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PersonRecordV1 {
    /// Stable person identifier.
    pub id: PersonId,
    /// Generated display name.
    pub name: String,
    /// Given name independent of later household naming.
    #[serde(default)]
    pub given_name: String,
    /// Surname received at initialization or birth.
    #[serde(default)]
    pub surname: String,
    /// Complete age at the scenario epoch.
    pub starting_age_years: u16,
    /// Complete age at the end of life or run.
    pub final_age_years: u64,
    /// Whether the person remains alive.
    pub alive: bool,
    /// Absolute death day when dead.
    pub death_day: Option<u64>,
    /// Absolute birth day for people born during the run.
    #[serde(default)]
    pub birth_day: Option<u64>,
    /// Stable parent identities, empty for scenario founders.
    #[serde(default)]
    pub parent_ids: Vec<PersonId>,
    /// Current household at the end of the run.
    #[serde(default)]
    pub household_id: Option<HouseholdId>,
    /// Current partner at the end of the run.
    #[serde(default)]
    pub partner_id: Option<PersonId>,
    /// Founder generation is zero; their children are generation one.
    #[serde(default)]
    pub generation: u16,
}

/// Inspectable household state at the end of a run.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HouseholdRecordV1 {
    /// Stable household identifier.
    pub id: HouseholdId,
    /// Human-readable household name.
    pub name: String,
    /// Surname inherited by children born into this household.
    pub surname: String,
    /// Current members in stable person order.
    pub member_ids: Vec<PersonId>,
    /// Absolute day on which the household formed.
    pub founded_day: u64,
    /// Absolute day on which the household became empty or extinct.
    pub dissolved_day: Option<u64>,
    /// Number of children born into this household.
    pub children_born: u16,
}
