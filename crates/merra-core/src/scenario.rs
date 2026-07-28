//! Human-authored scenario configuration.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::CalendarConfig;

/// Current supported scenario schema.
pub const SCENARIO_SCHEMA_V1: u32 = 1;

/// A complete schema-version-1 scenario.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScenarioV1 {
    /// Schema version, which must be `1`.
    pub schema_version: u32,
    /// Stable machine-readable scenario identifier.
    pub id: String,
    /// Human-readable scenario title.
    pub title: String,
    /// Scenario calendar.
    pub calendar: CalendarConfig,
}

impl ScenarioV1 {
    /// Validates invariants before a scenario enters the simulation.
    pub fn validate(&self) -> Result<(), ScenarioError> {
        if self.schema_version != SCENARIO_SCHEMA_V1 {
            return Err(ScenarioError::UnsupportedSchema {
                found: self.schema_version,
                supported: SCENARIO_SCHEMA_V1,
            });
        }
        if self.id.trim().is_empty() {
            return Err(ScenarioError::EmptyId);
        }
        if self.title.trim().is_empty() {
            return Err(ScenarioError::EmptyTitle);
        }
        if !self.calendar.is_valid() {
            return Err(ScenarioError::InvalidCalendar);
        }
        Ok(())
    }
}

/// Scenario validation failure.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum ScenarioError {
    /// The input uses a schema this build does not understand.
    #[error("unsupported scenario schema {found}; this build supports {supported}")]
    UnsupportedSchema {
        /// Schema found in the input.
        found: u32,
        /// Schema supported by this build.
        supported: u32,
    },
    /// The stable identifier is blank.
    #[error("scenario id must not be empty")]
    EmptyId,
    /// The display title is blank.
    #[error("scenario title must not be empty")]
    EmptyTitle,
    /// The calendar cannot advance.
    #[error("scenario calendar must contain at least one day per year")]
    InvalidCalendar,
}

#[cfg(test)]
mod tests {
    use super::{SCENARIO_SCHEMA_V1, ScenarioError, ScenarioV1};
    use crate::CalendarConfig;

    #[test]
    fn rejects_unknown_schema() {
        let scenario = ScenarioV1 {
            schema_version: SCENARIO_SCHEMA_V1 + 1,
            id: String::from("future"),
            title: String::from("Future"),
            calendar: CalendarConfig { days_per_year: 360 },
        };

        assert!(matches!(
            scenario.validate(),
            Err(ScenarioError::UnsupportedSchema { .. })
        ));
    }
}
