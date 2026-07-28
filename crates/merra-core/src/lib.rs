//! Bevy-independent domain types and deterministic data contracts for Merra.

mod calendar;
mod event;
mod ids;
mod output;
mod rng;
mod scenario;

pub use calendar::{CalendarConfig, CalendarError, SeasonConfigV1, SimDuration, SimTime};
pub use event::{EventKindV1, EventPayloadV1, WorldEventV1};
pub use ids::{EventId, HouseholdId, LocationId, PersonId};
pub use output::{
    HouseholdRecordV1, PersonRecordV1, RunManifestV1, SimulationSummaryV1, SourceVersionV1,
};
pub use rng::{RngDomain, rng_for_domain, seed_for_domain};
pub use scenario::{
    FamilyConfigV1, MortalityBandV1, PopulationConfigV1, SCENARIO_SCHEMA_V1, ScenarioError,
    ScenarioV1,
};

/// Foundation event schema used by time, season, and mortality-only runs.
pub const EVENT_SCHEMA_V1: u32 = 1;

/// Family event schema with households, partnerships, and births.
pub const EVENT_SCHEMA_V2: u32 = 2;

/// Current simulation-summary schema.
pub const SUMMARY_SCHEMA_V1: u32 = 1;

/// Current run-manifest schema.
pub const MANIFEST_SCHEMA_V1: u32 = 1;

/// Bevy version selected by the workspace.
pub const BEVY_VERSION: &str = "0.19.0";

/// Rust toolchain pinned by `rust-toolchain.toml`.
pub const RUST_TOOLCHAIN_VERSION: &str = "1.97.1";
