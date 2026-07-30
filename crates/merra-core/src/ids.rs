//! Stable identifiers that survive serialization and ECS reconstruction.

use serde::{Deserialize, Serialize};

macro_rules! stable_id {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(
            Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub u64);
    };
}

stable_id!(EventId, "Stable identifier for a historical event.");
stable_id!(PersonId, "Stable identifier for a person.");
stable_id!(HouseholdId, "Stable identifier for a household.");
stable_id!(ItemId, "Stable identifier for a durable movable item.");
stable_id!(LocationId, "Stable identifier for a location.");
stable_id!(RegionId, "Stable identifier for a coarse world region.");
stable_id!(
    FeatureId,
    "Stable identifier for a physical or mythic feature."
);
stable_id!(RouteId, "Stable identifier for a route between locations.");
stable_id!(
    PopulationId,
    "Stable identifier for an aggregate population."
);
stable_id!(LineageId, "Stable identifier for a biological lineage.");
stable_id!(CultureId, "Stable identifier for a culture.");
stable_id!(FaithId, "Stable identifier for a faith tradition.");
stable_id!(
    InstitutionId,
    "Stable identifier for a historical institution."
);
stable_id!(PolityId, "Stable identifier for a polity.");
