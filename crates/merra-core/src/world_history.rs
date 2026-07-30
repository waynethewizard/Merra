//! Portable aggregate-history, lineage, culture, faith, and lore contracts.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    CultureId, EventId, FaithId, FeatureId, InstitutionId, LineageId, LocationId, PolityId,
    PopulationId, SimTime,
};

/// Current aggregate-history schema.
pub const HISTORY_SCHEMA_V1: u32 = 1;

/// Data-defined biological pressures shared by every lineage.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LineagePhysiologyV1 {
    pub adult_mortality_multiplier_per_10_000: u16,
    pub physical_power_per_10_000: u16,
    pub movement_speed_per_10_000: u16,
    pub sustenance_demand_per_10_000: u16,
}

/// A biological lineage without cultural behavior.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LineageDefinitionV1 {
    pub id: LineageId,
    pub key: String,
    pub name: String,
    pub physiology: LineagePhysiologyV1,
}

/// Initial cultural process parameters.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CultureSeedV1 {
    pub key: String,
    pub name: String,
    pub ritual_days_per_year: u16,
    pub sacred_contribution_per_10_000: u16,
    pub institutional_preservation_per_10_000: u16,
    pub faith_transmission_per_10_000: u16,
}

/// Optional founding faith.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FaithSeedV1 {
    pub key: String,
    pub name: String,
    pub culture_key: String,
    pub founded_year: u32,
    pub source_motif_id: Option<String>,
    pub tags: Vec<String>,
    pub founding_institution: bool,
}

/// One initial population, with inherited lineage and learned culture separate.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FounderSeedV1 {
    pub lineage_id: LineageId,
    pub homeland_tag: String,
    pub culture: CultureSeedV1,
}

/// Authored voice used to interpret a future first-contact event.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LoreSeedV1 {
    pub title: String,
    pub text: String,
    pub source_culture_key: String,
    pub source_faith_key: Option<String>,
    pub confidence_per_10_000: u16,
}

/// Authored constraints for one coarse historical age.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HistoryConfigV1 {
    pub schema_version: u32,
    pub id: String,
    pub title: String,
    pub days_per_year: u16,
    pub years: u32,
    pub initial_population_per_cohort: u32,
    pub lineages: Vec<LineageDefinitionV1>,
    pub founders: Vec<FounderSeedV1>,
    pub faiths: Vec<FaithSeedV1>,
    pub contact_culture: CultureSeedV1,
    pub contact_lore: Vec<LoreSeedV1>,
    pub contact_navigation_threshold: u32,
}

impl HistoryConfigV1 {
    /// Validates portable rules before they enter Bevy.
    pub fn validate(&self) -> Result<(), HistoryError> {
        if self.schema_version != HISTORY_SCHEMA_V1 {
            return Err(HistoryError::UnsupportedSchema(self.schema_version));
        }
        if self.id.trim().is_empty() || self.title.trim().is_empty() {
            return Err(HistoryError::MissingIdentity);
        }
        if self.days_per_year == 0 || self.years == 0 || self.initial_population_per_cohort == 0 {
            return Err(HistoryError::EmptyDurationOrPopulation);
        }
        if self.lineages.is_empty() || self.founders.len() < 2 {
            return Err(HistoryError::MissingFounders);
        }
        let mut lineage_ids = BTreeSet::new();
        for lineage in &self.lineages {
            if lineage.key.trim().is_empty()
                || lineage.name.trim().is_empty()
                || !lineage_ids.insert(lineage.id)
            {
                return Err(HistoryError::InvalidLineage);
            }
            let physiology = &lineage.physiology;
            if physiology.adult_mortality_multiplier_per_10_000 == 0
                || physiology.physical_power_per_10_000 == 0
                || physiology.movement_speed_per_10_000 == 0
                || physiology.sustenance_demand_per_10_000 == 0
            {
                return Err(HistoryError::InvalidLineage);
            }
        }
        let mut culture_keys = BTreeSet::new();
        for founder in &self.founders {
            if !lineage_ids.contains(&founder.lineage_id) || founder.homeland_tag.trim().is_empty()
            {
                return Err(HistoryError::InvalidFounder);
            }
            let culture = &founder.culture;
            if culture.key.trim().is_empty()
                || culture.name.trim().is_empty()
                || !culture_keys.insert(culture.key.as_str())
                || culture.ritual_days_per_year > self.days_per_year
                || culture.sacred_contribution_per_10_000 > 10_000
            {
                return Err(HistoryError::InvalidCulture);
            }
        }
        if self.contact_culture.key.trim().is_empty()
            || self.contact_culture.name.trim().is_empty()
            || culture_keys.contains(self.contact_culture.key.as_str())
            || self.contact_culture.ritual_days_per_year > self.days_per_year
            || self.contact_culture.sacred_contribution_per_10_000 > 10_000
        {
            return Err(HistoryError::InvalidCulture);
        }
        culture_keys.insert(self.contact_culture.key.as_str());
        let mut faith_keys = BTreeSet::new();
        for faith in &self.faiths {
            if faith.key.trim().is_empty()
                || faith.name.trim().is_empty()
                || faith.culture_key.trim().is_empty()
                || !culture_keys.contains(faith.culture_key.as_str())
                || !faith_keys.insert(faith.key.as_str())
                || faith.founded_year > self.years
                || faith.tags.iter().any(|tag| tag.trim().is_empty())
                || (faith.founded_year > 0 && faith.source_motif_id.is_some())
                || faith
                    .source_motif_id
                    .as_ref()
                    .is_some_and(|motif| motif.trim().is_empty())
            {
                return Err(HistoryError::InvalidFaith);
            }
        }
        if self.contact_lore.is_empty()
            || self.contact_lore.iter().any(|claim| {
                claim.title.trim().is_empty()
                    || claim.text.trim().is_empty()
                    || !culture_keys.contains(claim.source_culture_key.as_str())
                    || claim
                        .source_faith_key
                        .as_ref()
                        .is_some_and(|key| !faith_keys.contains(key.as_str()))
                    || claim.confidence_per_10_000 > 10_000
            })
        {
            return Err(HistoryError::InvalidLore);
        }
        Ok(())
    }
}

/// One normalized membership share.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AffiliationShareV1<T> {
    pub id: T,
    pub parts_per_10_000: u16,
}

/// A lineage-aware aggregate population at the end of a run.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PopulationRecordV1 {
    pub id: PopulationId,
    pub name: String,
    pub location_id: LocationId,
    pub people: u32,
    pub founded_year: u32,
    pub lineage: Vec<AffiliationShareV1<LineageId>>,
    pub cultures: Vec<AffiliationShareV1<CultureId>>,
    pub faiths: Vec<AffiliationShareV1<FaithId>>,
}

/// An evolving learned culture.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CultureRecordV1 {
    pub id: CultureId,
    pub key: String,
    pub name: String,
    pub founded_year: u32,
    pub origin_event: EventId,
    pub ritual_days_per_year: u16,
    pub sacred_contribution_per_10_000: u16,
    pub institutional_preservation_per_10_000: u16,
    pub faith_transmission_per_10_000: u16,
}

/// An evolving faith tradition.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FaithRecordV1 {
    pub id: FaithId,
    pub key: String,
    pub name: String,
    pub founded_year: u32,
    pub origin_event: EventId,
    pub source_feature_id: Option<FeatureId>,
    pub parent_faith_id: Option<FaithId>,
}

/// A settlement's current state and historical bounds.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SettlementRecordV1 {
    pub location_id: LocationId,
    pub name: String,
    pub founded_year: u32,
    pub abandoned_year: Option<u32>,
    pub population: u32,
    pub founding_event: EventId,
}

/// A durable learned organization.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct InstitutionRecordV1 {
    pub id: InstitutionId,
    pub name: String,
    pub founded_year: u32,
    pub dissolved_year: Option<u32>,
    pub culture_id: CultureId,
    pub faith_id: Option<FaithId>,
    pub location_id: LocationId,
    pub founding_event: EventId,
}

/// A coarse political institution spanning one or more locations.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PolityRecordV1 {
    pub id: PolityId,
    pub name: String,
    pub founded_year: u32,
    pub dissolved_year: Option<u32>,
    pub culture_ids: Vec<CultureId>,
    pub location_ids: Vec<LocationId>,
    pub founding_event: EventId,
}

/// Anything that can be the subject of aggregate history.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "id", rename_all = "snake_case")]
pub enum HistoricalSubjectV1 {
    Population(PopulationId),
    Location(LocationId),
    Culture(CultureId),
    Faith(FaithId),
    Institution(InstitutionId),
    Polity(PolityId),
    Feature(FeatureId),
}

/// Stable macro-event categories.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HistoricalEventKindV1 {
    HistoryStarted,
    PopulationSeeded,
    SettlementFounded,
    PopulationMigrated,
    CultureFounded,
    FaithFounded,
    InstitutionFounded,
    PolityFounded,
    RouteOpened,
    SeaRouteOpened,
    FirstContact,
    PopulationsMixed,
    FaithSpread,
    FaithSchism,
    SettlementAbandoned,
    HistoryCompleted,
}

/// Typed details for one macro event.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HistoricalEventPayloadV1 {
    HistoryStarted {
        history_id: String,
        seed: u64,
    },
    PopulationSeeded {
        population_id: PopulationId,
        people: u32,
    },
    SettlementFounded {
        location_id: LocationId,
        name: String,
    },
    PopulationMigrated {
        population_id: PopulationId,
        from: LocationId,
        to: LocationId,
        people: u32,
    },
    CultureFounded {
        culture_id: CultureId,
        name: String,
    },
    FaithFounded {
        faith_id: FaithId,
        name: String,
    },
    InstitutionFounded {
        institution_id: InstitutionId,
        name: String,
    },
    PolityFounded {
        polity_id: PolityId,
        name: String,
    },
    RouteOpened {
        route_id: crate::RouteId,
        capability: String,
    },
    SeaRouteOpened {
        route_id: crate::RouteId,
    },
    FirstContact {
        populations: [PopulationId; 2],
    },
    PopulationsMixed {
        location_id: LocationId,
        lineages: Vec<AffiliationShareV1<LineageId>>,
    },
    FaithSpread {
        faith_id: FaithId,
        population_id: PopulationId,
    },
    FaithSchism {
        parent_id: FaithId,
        child_id: FaithId,
    },
    SettlementAbandoned {
        location_id: LocationId,
    },
    HistoryCompleted {
        elapsed_years: u32,
    },
}

/// One authoritative aggregate historical event.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HistoricalEventV1 {
    pub id: EventId,
    pub time: SimTime,
    pub kind: HistoricalEventKindV1,
    pub subjects: Vec<HistoricalSubjectV1>,
    pub location: Option<LocationId>,
    pub causes: Vec<EventId>,
    pub tags: Vec<String>,
    pub payload: HistoricalEventPayloadV1,
}

/// One deterministic but potentially biased interpretation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct LoreClaimV1 {
    pub id: u64,
    pub title: String,
    pub text: String,
    pub source_culture_id: CultureId,
    pub source_faith_id: Option<FaithId>,
    pub about_events: Vec<EventId>,
    pub confidence_per_10_000: u16,
}

/// Why a place is historically important.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ImportantPlaceV1 {
    pub location_id: LocationId,
    pub score: u32,
    pub reasons: Vec<String>,
    pub event_ids: Vec<EventId>,
}

/// Portable handoff from macro history to future detailed simulation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StartingRegionV1 {
    pub anchor_location_id: LocationId,
    pub settlement_ids: Vec<LocationId>,
    pub population_ids: Vec<PopulationId>,
    pub event_ids: Vec<EventId>,
    pub summary: String,
}

/// Reproducibility metadata for a historical run.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HistoryManifestV1 {
    pub schema_version: u32,
    pub history_id: String,
    pub history_hash: String,
    pub world_hash: String,
    pub seed: u64,
    pub years: u32,
}

/// Compact aggregate-history measurements.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct HistorySummaryV1 {
    pub schema_version: u32,
    pub history_id: String,
    pub seed: u64,
    pub elapsed_years: u32,
    pub total_population: u64,
    pub population_cohorts: usize,
    pub settlements: usize,
    pub cultures: usize,
    pub faiths: usize,
    pub institutions: usize,
    pub mixed_lineage_populations: usize,
    pub first_contact_year: Option<u32>,
    pub event_count: usize,
}

/// Invalid historical configuration or input graph.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum HistoryError {
    #[error("unsupported history schema {0}")]
    UnsupportedSchema(u32),
    #[error("history id and title must not be blank")]
    MissingIdentity,
    #[error("history duration, calendar, and population must be positive")]
    EmptyDurationOrPopulation,
    #[error("history requires at least two founder populations and one lineage")]
    MissingFounders,
    #[error("lineage definitions must be unique, named, and positive")]
    InvalidLineage,
    #[error("founder lineages and homeland tags must resolve")]
    InvalidFounder,
    #[error("culture process parameters are invalid")]
    InvalidCulture,
    #[error("founding faith is invalid")]
    InvalidFaith,
    #[error("contact lore is invalid")]
    InvalidLore,
    #[error("place graph cannot satisfy all configured founder homelands")]
    InsufficientSeedLocations,
    #[error("affiliation shares must sum to 10,000")]
    InvalidAffiliationShares,
}
