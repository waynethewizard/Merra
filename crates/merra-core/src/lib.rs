//! Bevy-independent domain types and deterministic data contracts for Merra.

mod calendar;
mod event;
mod ids;
mod local_history;
mod output;
mod rng;
mod scenario;
mod world;
mod world_history;

pub use calendar::{CalendarConfig, CalendarError, SeasonConfigV1, SimDuration, SimTime};
pub use event::{EventKindV1, EventPayloadV1, WorldEventV1};
pub use ids::{
    CultureId, EventId, FaithId, FeatureId, HouseholdId, InstitutionId, LineageId, LocationId,
    PersonId, PolityId, PopulationId, RegionId, RouteId,
};
pub use local_history::{
    HouseholdHistoricalContextV1, LOCAL_HISTORY_SCHEMA_V1, LOCAL_PLAYBACK_SCHEMA_V1,
    LocalConnectionV1, LocalHistoryConfigError, LocalHistoryConfigV1, LocalHistoryManifestV1,
    LocalHistoryPlaybackV1, LocalHistoryReportV1, LocalHistorySummaryV1, LocalPlaybackEventV1,
    LocalPlaybackPersonV1, LocalSettlementRecordV1, PopulationAllocationV1, RegionalHistoryV1,
    ResidenceDecisionV1, ResidenceReasonV1,
};
pub use output::{
    HouseholdRecordV1, PersonRecordV1, RunManifestV1, SimulationSummaryV1, SourceVersionV1,
};
pub use rng::{RngDomain, rng_for_domain, seed_for_domain};
pub use scenario::{
    FamilyConfigV1, MortalityBandV1, PopulationConfigV1, SCENARIO_SCHEMA_V1, ScenarioError,
    ScenarioV1,
};
pub use world::{
    BiomeV1, CellResourceV1, CoordinateV1, FeatureKindV1, GenerationPassV1, LandformV1,
    LocationRecordV1, MythicMotifConfigV1, PlaceAffordanceV1, PlaceGraphV1, RouteKindV1,
    RouteRecordV1, SurfaceCellV1, SurfaceWorldV1, WORLD_GENESIS_SCHEMA_V1, WorldFeatureV1,
    WorldGenesisConfigV1, WorldGenesisError, WorldGenesisManifestV1, WorldGenesisSummaryV1,
};
pub use world_history::{
    AffiliationShareV1, CultureRecordV1, CultureSeedV1, FaithRecordV1, FaithSeedV1, FounderSeedV1,
    HISTORY_SCHEMA_V1, HistoricalEventKindV1, HistoricalEventPayloadV1, HistoricalEventV1,
    HistoricalSubjectV1, HistoryConfigV1, HistoryError, HistoryManifestV1, HistorySummaryV1,
    ImportantPlaceV1, InstitutionRecordV1, LineageDefinitionV1, LineagePhysiologyV1, LoreClaimV1,
    LoreSeedV1, PolityRecordV1, PopulationRecordV1, SettlementRecordV1, StartingRegionV1,
};

/// Foundation event schema used by time, season, and mortality-only runs.
pub const EVENT_SCHEMA_V1: u32 = 1;

/// Family event schema with households, partnerships, and births.
pub const EVENT_SCHEMA_V2: u32 = 2;

/// Local-history event schema with household residence and movement evidence.
pub const EVENT_SCHEMA_V3: u32 = 3;

/// Current simulation-summary schema.
pub const SUMMARY_SCHEMA_V1: u32 = 1;

/// Current run-manifest schema.
pub const MANIFEST_SCHEMA_V1: u32 = 1;

/// Bevy version selected by the workspace.
pub const BEVY_VERSION: &str = "0.19.0";

/// Rust toolchain pinned by `rust-toolchain.toml`.
pub const RUST_TOOLCHAIN_VERSION: &str = "1.97.1";
