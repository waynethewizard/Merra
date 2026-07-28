//! Versioned output types written by the headless runner.

use serde::{Deserialize, Serialize};

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
    /// Number of structured events emitted.
    pub event_count: usize,
}
