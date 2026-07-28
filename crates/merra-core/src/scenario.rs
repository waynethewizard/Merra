//! Human-authored scenario configuration.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{CalendarConfig, CalendarError};

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
    /// Initial population and mortality rules.
    pub population: PopulationConfigV1,
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
        self.calendar.validate()?;
        self.population.validate()?;
        Ok(())
    }
}

/// Initial population and its data-driven mortality table.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PopulationConfigV1 {
    /// Number of people present at the scenario epoch.
    pub initial_people: u32,
    /// Minimum starting age in complete years.
    pub minimum_starting_age: u16,
    /// Maximum starting age in complete years.
    pub maximum_starting_age: u16,
    /// Ordered inclusive age bands used for annual mortality checks.
    pub mortality_bands: Vec<MortalityBandV1>,
}

impl PopulationConfigV1 {
    fn validate(&self) -> Result<(), ScenarioError> {
        if self.minimum_starting_age > self.maximum_starting_age {
            return Err(ScenarioError::InvalidStartingAges);
        }
        if self.initial_people == 0 {
            return Ok(());
        }
        if self.mortality_bands.is_empty() {
            return Err(ScenarioError::MissingMortalityBands);
        }

        let mut previous = None;
        for band in &self.mortality_bands {
            if band.annual_deaths_per_10_000 > 10_000 {
                return Err(ScenarioError::InvalidMortalityRate {
                    through_age: band.through_age,
                    rate: band.annual_deaths_per_10_000,
                });
            }
            if previous.is_some_and(|age| band.through_age <= age) {
                return Err(ScenarioError::UnorderedMortalityBands);
            }
            previous = Some(band.through_age);
        }
        if previous != Some(u16::MAX) {
            return Err(ScenarioError::IncompleteMortalityBands);
        }
        Ok(())
    }

    /// Returns the annual threshold for a complete age.
    #[must_use]
    pub fn annual_mortality_per_10_000(&self, age_years: u64) -> u16 {
        self.mortality_bands
            .iter()
            .find(|band| age_years <= u64::from(band.through_age))
            .map_or(10_000, |band| band.annual_deaths_per_10_000)
    }
}

/// An inclusive age band with an integer annual mortality threshold.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct MortalityBandV1 {
    /// Inclusive maximum complete age for this band.
    pub through_age: u16,
    /// Annual deaths per 10,000 living people in this band.
    pub annual_deaths_per_10_000: u16,
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
    /// The calendar is internally inconsistent.
    #[error(transparent)]
    Calendar(#[from] CalendarError),
    /// The initial age range is inverted.
    #[error("minimum starting age must not exceed maximum starting age")]
    InvalidStartingAges,
    /// A non-empty population needs mortality rules.
    #[error("a non-empty population requires at least one mortality band")]
    MissingMortalityBands,
    /// A mortality threshold exceeds certainty.
    #[error("mortality rate {rate} for age band through {through_age} exceeds 10,000 per 10,000")]
    InvalidMortalityRate {
        /// Inclusive maximum age of the invalid band.
        through_age: u16,
        /// Invalid threshold.
        rate: u16,
    },
    /// Mortality bands overlap or are not strictly increasing.
    #[error("mortality bands must have strictly increasing inclusive ages")]
    UnorderedMortalityBands,
    /// The mortality table would not cover arbitrarily old people.
    #[error("the final mortality band must end at 65535")]
    IncompleteMortalityBands,
}

#[cfg(test)]
mod tests {
    use super::{
        MortalityBandV1, PopulationConfigV1, SCENARIO_SCHEMA_V1, ScenarioError, ScenarioV1,
    };
    use crate::{CalendarConfig, SeasonConfigV1};

    fn calendar() -> CalendarConfig {
        CalendarConfig {
            days_per_year: 360,
            seasons: vec![SeasonConfigV1 {
                id: String::from("year"),
                name: String::from("Year"),
                days: 360,
            }],
        }
    }

    #[test]
    fn rejects_unknown_schema() {
        let scenario = ScenarioV1 {
            schema_version: SCENARIO_SCHEMA_V1 + 1,
            id: String::from("future"),
            title: String::from("Future"),
            calendar: calendar(),
            population: PopulationConfigV1 {
                initial_people: 0,
                minimum_starting_age: 0,
                maximum_starting_age: 0,
                mortality_bands: Vec::new(),
            },
        };

        assert!(matches!(
            scenario.validate(),
            Err(ScenarioError::UnsupportedSchema { .. })
        ));
    }

    #[test]
    fn rejects_invalid_mortality_tables() {
        let populated = |mortality_bands| ScenarioV1 {
            schema_version: SCENARIO_SCHEMA_V1,
            id: String::from("mortality-test"),
            title: String::from("Mortality Test"),
            calendar: calendar(),
            population: PopulationConfigV1 {
                initial_people: 1,
                minimum_starting_age: 0,
                maximum_starting_age: 70,
                mortality_bands,
            },
        };

        let invalid_rate = populated(vec![MortalityBandV1 {
            through_age: u16::MAX,
            annual_deaths_per_10_000: 10_001,
        }]);
        assert!(matches!(
            invalid_rate.validate(),
            Err(ScenarioError::InvalidMortalityRate { .. })
        ));

        let unordered = populated(vec![
            MortalityBandV1 {
                through_age: 70,
                annual_deaths_per_10_000: 100,
            },
            MortalityBandV1 {
                through_age: 70,
                annual_deaths_per_10_000: 200,
            },
        ]);
        assert_eq!(
            unordered.validate(),
            Err(ScenarioError::UnorderedMortalityBands)
        );

        let incomplete = populated(vec![MortalityBandV1 {
            through_age: 100,
            annual_deaths_per_10_000: 100,
        }]);
        assert_eq!(
            incomplete.validate(),
            Err(ScenarioError::IncompleteMortalityBands)
        );
    }
}
