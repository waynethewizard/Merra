//! Durable item identity, provenance, ownership, and custody contracts.

use serde::{Deserialize, Serialize};

use crate::{EventId, HouseholdId, InstitutionId, ItemId, LocationId, PersonId, PolityId};

/// A legal owner. Ownership is independent from physical custody.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "id", rename_all = "snake_case")]
pub enum PropertyOwnerV1 {
    Person(PersonId),
    Household(HouseholdId),
    Institution(InstitutionId),
    Settlement(LocationId),
    Polity(PolityId),
}

/// The person, group, or place physically holding an item.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "id", rename_all = "snake_case")]
pub enum ItemCustodyV1 {
    Person(PersonId),
    Household(HouseholdId),
    Institution(InstitutionId),
    AtLocation(LocationId),
    Unknown,
}

/// How an earlier item contributed to a descendant item.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemSourceRoleV1 {
    Material,
    Component,
    Pattern,
}

/// One typed edge in an item's immutable provenance graph.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ItemSourceV1 {
    pub item_id: ItemId,
    pub role: ItemSourceRoleV1,
}

/// Current authoritative lifecycle state.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemStatusV1 {
    Active,
    Lost,
    Transformed,
    Destroyed,
    Consumed,
}

/// Why legal ownership changed.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnershipTransferReasonV1 {
    HouseholdFormation,
    Inheritance,
    DebtSettlement,
    Gift,
    Recovery,
}

/// Data-defined behavior shared by one class of durable items.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ItemArchetypeV1 {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub initially_distributed: bool,
    pub work_tag: String,
    pub productivity_per_10_000: u16,
    pub wear_per_use: u16,
    pub repair_below: u16,
    pub repair_amount: u16,
    pub maximum_repairs: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rework_into: Option<String>,
}

/// Opt-in item-history rules.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ItemConfigV1 {
    pub enabled: bool,
    /// Number of each archetype introduced into every initial household.
    pub initial_items_per_household: u16,
    /// Founders carry one prior household's item into their new household.
    #[serde(default)]
    pub household_formation_contributions: bool,
    #[serde(default)]
    pub archetypes: Vec<ItemArchetypeV1>,
}

/// Final inspectable state plus immutable lineage identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ItemRecordV1 {
    pub id: ItemId,
    pub archetype_id: String,
    pub name: String,
    pub introduced_day: u64,
    pub introduction_event_id: EventId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<ItemSourceV1>,
    pub lineage_generation: u16,
    pub condition_per_10_000: u16,
    pub repairs: u16,
    pub status: ItemStatusV1,
    pub owner: PropertyOwnerV1,
    pub custody: ItemCustodyV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_location_id: Option<LocationId>,
}
