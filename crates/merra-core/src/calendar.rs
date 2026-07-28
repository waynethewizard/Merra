//! Simulation calendar value types.

use serde::{Deserialize, Serialize};

/// Configures the calendar used by a scenario.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CalendarConfig {
    /// Number of days in one scenario year.
    pub days_per_year: u16,
}

impl CalendarConfig {
    /// Returns whether this calendar can advance safely.
    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.days_per_year > 0
    }
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
    pub const fn year(self, calendar: CalendarConfig) -> u64 {
        self.day / calendar.days_per_year as u64
    }

    /// Returns the zero-based day within the current year.
    #[must_use]
    pub const fn day_of_year(self, calendar: CalendarConfig) -> u16 {
        (self.day % calendar.days_per_year as u64) as u16
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
    pub const fn from_years(years: u32, calendar: CalendarConfig) -> Self {
        Self {
            days: years as u64 * calendar.days_per_year as u64,
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
    use super::{CalendarConfig, SimDuration, SimTime};

    #[test]
    fn calendar_conversion_is_zero_based() {
        let calendar = CalendarConfig { days_per_year: 360 };
        let time = SimTime::EPOCH.saturating_add(SimDuration::from_days(361));

        assert_eq!(time.year(calendar), 1);
        assert_eq!(time.day_of_year(calendar), 1);
    }
}
