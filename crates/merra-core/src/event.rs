//! Structured historical events.

use serde::{Deserialize, Serialize};

use crate::{
    EventId, HouseholdId, InstitutionId, ItemCustodyV1, ItemId, ItemSourceV1, LocationId,
    OwnershipTransferReasonV1, PersonId, PolityId, PropertyOwnerV1, ResidenceReasonV1, RouteId,
    SimTime,
};

/// Anything that can be indexed as a subject of detailed history.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "id", rename_all = "snake_case")]
pub enum WorldSubjectV1 {
    Person(PersonId),
    Household(HouseholdId),
    Item(ItemId),
    Location(LocationId),
    Institution(InstitutionId),
    Polity(PolityId),
}

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
    /// A household completed one significant unit of seasonal work.
    HouseholdWorkCompleted,
    /// A pre-existing durable object entered detailed history.
    ItemIntroduced,
    /// A durable object contributed to meaningful work and incurred wear.
    ItemUsed,
    /// Maintenance restored an item's condition without replacing its identity.
    ItemRepaired,
    /// One or more source objects became descendant objects.
    ItemTransformed,
    /// Legal title to an object changed.
    ItemOwnershipTransferred,
    /// Physical custody of an object changed.
    ItemCustodyTransferred,
    /// An object moved because its custodian household moved.
    ItemRelocated,
    /// An object's physical whereabouts became unknown.
    ItemLost,
    /// A lost object returned to known custody.
    ItemRecovered,
    /// An object ceased to exist without a descendant.
    ItemDestroyed,
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
    /// Inspectable labor consequence from household work and tool condition.
    HouseholdWorkCompleted {
        household_id: HouseholdId,
        item_id: ItemId,
        work_tag: String,
        base_labor: u32,
        effective_labor: u32,
    },
    /// A pre-existing item entered the detailed simulation boundary.
    ItemIntroduced {
        item_id: ItemId,
        archetype_id: String,
        name: String,
        owner: PropertyOwnerV1,
        custody: ItemCustodyV1,
    },
    /// One significant unit of work used a durable item.
    ItemUsed {
        item_id: ItemId,
        work_tag: String,
        productivity_per_10_000: u16,
        condition_before_per_10_000: u16,
        condition_after_per_10_000: u16,
    },
    /// Maintenance restored the same durable identity.
    ItemRepaired {
        item_id: ItemId,
        condition_before_per_10_000: u16,
        condition_after_per_10_000: u16,
        repair_number: u16,
    },
    /// Physical sources were retired into newly identified descendants.
    ItemTransformed {
        source_item_ids: Vec<ItemId>,
        output_item_ids: Vec<ItemId>,
        output_sources: Vec<Vec<ItemSourceV1>>,
    },
    /// Legal ownership changed without implying physical movement.
    ItemOwnershipTransferred {
        item_id: ItemId,
        from: PropertyOwnerV1,
        to: PropertyOwnerV1,
        reason: OwnershipTransferReasonV1,
    },
    /// Custody changed independently from legal ownership.
    ItemCustodyTransferred {
        item_id: ItemId,
        from: ItemCustodyV1,
        to: ItemCustodyV1,
    },
    /// Known physical location changed while custody remained stable.
    ItemRelocated {
        item_id: ItemId,
        from: LocationId,
        to: LocationId,
        route_ids: Vec<RouteId>,
    },
    /// The item no longer had known physical custody.
    ItemLost {
        item_id: ItemId,
        previous_custody: ItemCustodyV1,
    },
    /// A lost item returned to known custody.
    ItemRecovered {
        item_id: ItemId,
        custody: ItemCustodyV1,
    },
    /// An item was destroyed without producing descendants.
    ItemDestroyed { item_id: ItemId },
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
    /// Stable non-person identities directly involved in this event.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subjects: Vec<WorldSubjectV1>,
    /// Optional location of the event.
    pub location: Option<LocationId>,
    /// Earlier events that causally contributed to this event.
    pub causes: Vec<EventId>,
    /// Searchable, non-authoritative labels.
    pub tags: Vec<String>,
    /// Typed event-specific details.
    pub payload: EventPayloadV1,
}
