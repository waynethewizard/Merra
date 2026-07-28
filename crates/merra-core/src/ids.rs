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
stable_id!(LocationId, "Stable identifier for a location.");
