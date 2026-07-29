//! Portable physical-world and place-graph contracts.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{FeatureId, LocationId, RegionId, RouteId};

/// Current physical-world schema.
pub const WORLD_GENESIS_SCHEMA_V1: u32 = 1;

/// Authored constraints for one deterministic surface world.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorldGenesisConfigV1 {
    /// Schema version, currently `1`.
    pub schema_version: u32,
    /// Stable template identifier.
    pub id: String,
    /// Human-readable title.
    pub title: String,
    /// Number of square regions from west to east.
    pub width: u16,
    /// Number of square regions from north to south.
    pub height: u16,
    /// Number of tectonic seed plates.
    pub plate_count: u16,
    /// Desired land coverage in parts per 10,000.
    pub land_fraction_per_10_000: u16,
    /// Desired island share of total land in parts per 10,000.
    pub island_land_fraction_per_10_000: u16,
    /// Minimum ocean regions between island and continent.
    pub island_separation: u16,
    /// Number of candidate habitable places.
    pub place_count: u16,
    /// Ambiguous motif families placed after physical generation.
    pub mythic_motifs: Vec<MythicMotifConfigV1>,
}

impl WorldGenesisConfigV1 {
    /// Validates generator bounds before any work occurs.
    pub fn validate(&self) -> Result<(), WorldGenesisError> {
        if self.schema_version != WORLD_GENESIS_SCHEMA_V1 {
            return Err(WorldGenesisError::UnsupportedSchema(self.schema_version));
        }
        if self.id.trim().is_empty() || self.title.trim().is_empty() {
            return Err(WorldGenesisError::MissingIdentity);
        }
        if !(32..=256).contains(&self.width) || !(24..=192).contains(&self.height) {
            return Err(WorldGenesisError::InvalidDimensions);
        }
        if !(4..=64).contains(&self.plate_count) {
            return Err(WorldGenesisError::InvalidPlateCount);
        }
        if !(2_500..=7_500).contains(&self.land_fraction_per_10_000) {
            return Err(WorldGenesisError::InvalidLandFraction);
        }
        if !(300..=1_500).contains(&self.island_land_fraction_per_10_000) {
            return Err(WorldGenesisError::InvalidIslandFraction);
        }
        if self.island_separation < 4 || self.place_count < 8 {
            return Err(WorldGenesisError::InsufficientWorldEvidence);
        }
        if self.mythic_motifs.is_empty() {
            return Err(WorldGenesisError::MissingMythicMotifs);
        }
        let mut ids = BTreeSet::new();
        for motif in &self.mythic_motifs {
            if motif.id.trim().is_empty()
                || motif.name.trim().is_empty()
                || motif.count == 0
                || !ids.insert(&motif.id)
            {
                return Err(WorldGenesisError::InvalidMythicMotif);
            }
        }
        Ok(())
    }
}

/// One authored but unexplained prehuman motif.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MythicMotifConfigV1 {
    /// Stable motif key.
    pub id: String,
    /// Display name used by atlases and later cultures.
    pub name: String,
    /// Number of features requested.
    pub count: u16,
}

/// Integer coordinate on a coarse square world.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CoordinateV1 {
    /// West-to-east coordinate.
    pub x: u16,
    /// North-to-south coordinate.
    pub y: u16,
}

/// Broad physical shape of a region.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LandformV1 {
    /// Open ocean.
    Ocean,
    /// Low coastal land.
    Coast,
    /// Low inland land.
    Lowland,
    /// Elevated rolling land.
    Highland,
    /// High mountain terrain.
    Mountain,
    /// Inland drainage sink.
    Lake,
}

/// Climate-and-terrain expression used by the canonical surface generator.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BiomeV1 {
    Ocean,
    Lake,
    Tundra,
    BorealForest,
    TemperateForest,
    Grassland,
    Wetland,
    Desert,
    Alpine,
}

/// A resource occurrence described by a setting-defined key and relative amount.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CellResourceV1 {
    /// Stable setting-defined resource key.
    pub resource: String,
    /// Relative availability in parts per 10,000.
    pub amount_per_10_000: u16,
}

/// Complete inspectable state of one coarse surface region.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SurfaceCellV1 {
    pub id: RegionId,
    pub coordinate: CoordinateV1,
    pub plate: u16,
    pub elevation: i16,
    pub temperature: i16,
    pub precipitation: u16,
    pub landform: LandformV1,
    pub biome: BiomeV1,
    pub flow_to: Option<RegionId>,
    pub drainage: u32,
    pub river: bool,
    pub island: bool,
    pub habitability: u16,
    pub resources: Vec<CellResourceV1>,
    pub feature_ids: Vec<FeatureId>,
}

/// Physical or ambiguous feature category.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureKindV1 {
    MountainRange,
    River,
    Watershed,
    MythicTrace { motif_id: String },
}

/// A named feature spanning one or more regions.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorldFeatureV1 {
    pub id: FeatureId,
    pub name: String,
    pub kind: FeatureKindV1,
    pub regions: Vec<RegionId>,
    pub description: String,
}

/// Generic historical affordance compiled from a setting-specific environment.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlaceAffordanceV1 {
    /// Stable capability key such as `food`, `fresh_water`, or `navigation`.
    pub id: String,
    /// Relative strength in parts per 10,000.
    pub value_per_10_000: u16,
}

/// One location available to a setting-independent history simulation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LocationRecordV1 {
    pub id: LocationId,
    pub name: String,
    /// Optional source region; absent for non-surface settings.
    pub region: Option<RegionId>,
    pub tags: Vec<String>,
    pub carrying_capacity: u32,
    pub hazard_per_10_000: u16,
    pub affordances: Vec<PlaceAffordanceV1>,
    pub feature_ids: Vec<FeatureId>,
}

/// Generic route category.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteKindV1 {
    Land,
    River,
    Sea,
    Abstract,
}

/// One possible connection between generic places.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct RouteRecordV1 {
    pub id: RouteId,
    pub endpoints: [LocationId; 2],
    pub kind: RouteKindV1,
    pub travel_cost: u32,
    pub capacity: u32,
    /// Locked routes require history to develop the named capability.
    pub locked: bool,
    pub required_capability: Option<String>,
}

/// Setting-independent geography consumed by macro-history.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PlaceGraphV1 {
    pub locations: Vec<LocationRecordV1>,
    pub routes: Vec<RouteRecordV1>,
}

/// One deterministic generation stage with inspectable hashes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GenerationPassV1 {
    pub name: String,
    pub input_hash: String,
    pub output_hash: String,
}

/// Complete generated surface state.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SurfaceWorldV1 {
    pub schema_version: u32,
    pub template_id: String,
    pub title: String,
    pub seed: u64,
    pub width: u16,
    pub height: u16,
    pub cells: Vec<SurfaceCellV1>,
    pub features: Vec<WorldFeatureV1>,
    pub places: PlaceGraphV1,
    pub passes: Vec<GenerationPassV1>,
}

/// Reproducibility metadata for a world-generation run.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorldGenesisManifestV1 {
    pub schema_version: u32,
    pub template_id: String,
    pub template_hash: String,
    pub world_hash: String,
    pub seed: u64,
    pub generator_version: String,
}

/// Compact geography measurements for cohorts and review.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct WorldGenesisSummaryV1 {
    pub schema_version: u32,
    pub template_id: String,
    pub seed: u64,
    pub regions: usize,
    pub land_regions: usize,
    pub island_regions: usize,
    pub river_regions: usize,
    pub biome_count: usize,
    pub feature_count: usize,
    pub location_count: usize,
    pub route_count: usize,
    pub locked_sea_routes: usize,
}

/// Invalid world template.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum WorldGenesisError {
    #[error("unsupported world-genesis schema {0}")]
    UnsupportedSchema(u32),
    #[error("world template id and title must not be blank")]
    MissingIdentity,
    #[error("world dimensions must be between 32x24 and 256x192")]
    InvalidDimensions,
    #[error("plate count must be between 4 and 64")]
    InvalidPlateCount,
    #[error("land fraction must be between 2,500 and 7,500 per 10,000")]
    InvalidLandFraction,
    #[error("island fraction must be between 300 and 1,500 per 10,000")]
    InvalidIslandFraction,
    #[error("world requires a separated island and at least eight places")]
    InsufficientWorldEvidence,
    #[error("at least one mythic motif is required")]
    MissingMythicMotifs,
    #[error("mythic motif ids and names must be unique and nonempty with positive counts")]
    InvalidMythicMotif,
}
