//! Simulation calendar value types.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Configures the calendar used by a scenario.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CalendarConfig {
    /// Number of days in one scenario year.
    pub days_per_year: u16,
    /// Ordered named seasons whose lengths exactly fill one year.
    pub seasons: Vec<SeasonConfigV1>,
}

impl CalendarConfig {
    /// Validates the calendar's lengths and stable season identities.
    pub fn validate(&self) -> Result<(), CalendarError> {
        if self.days_per_year == 0 {
            return Err(CalendarError::EmptyYear);
        }
        if self.seasons.is_empty() {
            return Err(CalendarError::NoSeasons);
        }

        let mut total_days = 0_u32;
        for (index, season) in self.seasons.iter().enumerate() {
            if season.id.trim().is_empty() {
                return Err(CalendarError::EmptySeasonId { index });
            }
            if season.name.trim().is_empty() {
                return Err(CalendarError::EmptySeasonName {
                    id: season.id.clone(),
                });
            }
            if season.days == 0 {
                return Err(CalendarError::EmptySeason {
                    id: season.id.clone(),
                });
            }
            if self.seasons[..index]
                .iter()
                .any(|previous| previous.id == season.id)
            {
                return Err(CalendarError::DuplicateSeasonId {
                    id: season.id.clone(),
                });
            }
            total_days = total_days.saturating_add(u32::from(season.days));
        }

        if total_days != u32::from(self.days_per_year) {
            return Err(CalendarError::SeasonLengthMismatch {
                year_days: self.days_per_year,
                season_days: total_days,
            });
        }
        Ok(())
    }

    /// Returns the season containing an absolute day.
    #[must_use]
    pub fn season_at_day(&self, day: u64) -> Option<&SeasonConfigV1> {
        let day_of_year = self.day_of_year(day)?;
        let mut end = 0_u16;
        self.seasons.iter().find(|season| {
            end = end.saturating_add(season.days);
            day_of_year < end
        })
    }

    /// Returns the season beginning on an exact boundary and its zero-based year.
    #[must_use]
    pub fn season_starting_at_day(&self, day: u64) -> Option<(u64, &SeasonConfigV1)> {
        let day_of_year = self.day_of_year(day)?;
        let mut start = 0_u16;
        for season in &self.seasons {
            if day_of_year == start {
                return Some((day / u64::from(self.days_per_year), season));
            }
            start = start.saturating_add(season.days);
        }
        None
    }

    /// Returns the positive duration from an absolute day to the next boundary.
    #[must_use]
    pub fn days_until_next_season(&self, day: u64) -> Option<u64> {
        let day_of_year = self.day_of_year(day)?;
        let mut end = 0_u16;
        for season in &self.seasons {
            end = end.saturating_add(season.days);
            if day_of_year < end {
                return Some(u64::from(end - day_of_year));
            }
        }
        None
    }

    fn day_of_year(&self, day: u64) -> Option<u16> {
        (self.days_per_year > 0).then(|| (day % u64::from(self.days_per_year)) as u16)
    }
}

/// A stable named season and its duration.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SeasonConfigV1 {
    /// Stable machine-readable season identifier.
    pub id: String,
    /// Human-readable season name.
    pub name: String,
    /// Number of days in the season.
    pub days: u16,
}

/// Calendar configuration failure.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum CalendarError {
    /// A year cannot contain zero days.
    #[error("scenario calendar must contain at least one day per year")]
    EmptyYear,
    /// Named seasons are required.
    #[error("scenario calendar must contain at least one named season")]
    NoSeasons,
    /// A season ID is blank.
    #[error("season at index {index} has an empty stable id")]
    EmptySeasonId {
        /// Position of the invalid season.
        index: usize,
    },
    /// A season display name is blank.
    #[error("season `{id}` has an empty display name")]
    EmptySeasonName {
        /// Stable identity of the invalid season.
        id: String,
    },
    /// A season cannot contain zero days.
    #[error("season `{id}` must contain at least one day")]
    EmptySeason {
        /// Stable identity of the invalid season.
        id: String,
    },
    /// Season IDs must be unique.
    #[error("season id `{id}` appears more than once")]
    DuplicateSeasonId {
        /// Repeated stable season identity.
        id: String,
    },
    /// Season lengths do not fill the configured year.
    #[error("season lengths total {season_days} days but the calendar year contains {year_days}")]
    SeasonLengthMismatch {
        /// Configured number of days in a year.
        year_days: u16,
        /// Sum of configured season lengths.
        season_days: u32,
    },
}

/// Absolute simulation time measured in days from the scenario epoch.
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct SimTime {
    day: u64,
}

impl SimTime {
    /// The first instant in a scenario.
    pub const EPOCH: Self = Self { day: 0 };

    /// Constructs a time from an absolute day.
    #[must_use]
    pub const fn from_day(day: u64) -> Self {
        Self { day }
    }

    /// Returns the absolute day.
    #[must_use]
    pub const fn day(self) -> u64 {
        self.day
    }

    /// Advances by a duration, saturating at the representable limit.
    #[must_use]
    pub const fn saturating_add(self, duration: SimDuration) -> Self {
        Self {
            day: self.day.saturating_add(duration.days),
        }
    }

    /// Returns the zero-based year for a calendar.
    #[must_use]
    pub const fn year(self, days_per_year: u16) -> u64 {
        self.day / days_per_year as u64
    }

    /// Returns the zero-based day within the current year.
    #[must_use]
    pub const fn day_of_year(self, days_per_year: u16) -> u16 {
        (self.day % days_per_year as u64) as u16
    }
}

/// A simulation duration measured in days.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct SimDuration {
    days: u64,
}

impl SimDuration {
    /// Constructs a duration from days.
    #[must_use]
    pub const fn from_days(days: u64) -> Self {
        Self { days }
    }

    /// Constructs a duration from years in the supplied calendar.
    #[must_use]
    pub const fn from_years(years: u32, days_per_year: u16) -> Self {
        Self {
            days: years as u64 * days_per_year as u64,
        }
    }

    /// Returns the number of days.
    #[must_use]
    pub const fn days(self) -> u64 {
        self.days
    }
}

#[cfg(test)]
mod tests {
    use super::{CalendarConfig, CalendarError, SeasonConfigV1, SimDuration, SimTime};

    fn calendar() -> CalendarConfig {
        CalendarConfig {
            days_per_year: 360,
            seasons: vec![
                SeasonConfigV1 {
                    id: String::from("thaw"),
                    name: String::from("Thaw"),
                    days: 60,
                },
                SeasonConfigV1 {
                    id: String::from("highsun"),
                    name: String::from("Highsun"),
                    days: 120,
                },
                SeasonConfigV1 {
                    id: String::from("emberfall"),
                    name: String::from("Emberfall"),
                    days: 180,
                },
            ],
        }
    }

    #[test]
    fn calendar_conversion_is_zero_based() {
        let calendar = calendar();
        let time = SimTime::EPOCH.saturating_add(SimDuration::from_days(361));

        assert_eq!(time.year(calendar.days_per_year), 1);
        assert_eq!(time.day_of_year(calendar.days_per_year), 1);
    }

    #[test]
    fn season_boundaries_are_data_driven() {
        let calendar = calendar();

        assert_eq!(calendar.validate(), Ok(()));
        assert_eq!(
            calendar.season_at_day(59).map(|season| season.id.as_str()),
            Some("thaw")
        );
        assert_eq!(calendar.days_until_next_season(59), Some(1));
        assert_eq!(
            calendar
                .season_starting_at_day(60)
                .map(|(year, season)| (year, season.id.as_str())),
            Some((0, "highsun"))
        );
        assert_eq!(
            calendar
                .season_starting_at_day(360)
                .map(|(year, season)| (year, season.id.as_str())),
            Some((1, "thaw"))
        );
    }

    #[test]
    fn season_lengths_must_fill_the_year() {
        let mut calendar = calendar();
        calendar.seasons[0].days = 59;

        assert!(matches!(
            calendar.validate(),
            Err(CalendarError::SeasonLengthMismatch { .. })
        ));
    }
}
