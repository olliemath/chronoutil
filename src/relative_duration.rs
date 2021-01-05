//! Implements a RelativeDuration extending Chrono's Duration to shift by months and years.
use core::ops::{Add, Div, Mul, Neg, Sub};
use std::time::Duration as StdDuration;

use chrono::{Date, DateTime, Duration, NaiveDate, NaiveDateTime, TimeZone};

use super::delta::shift_months;

/// Relative time duration extending Chrono's Duration.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct RelativeDuration {
    months: i32, // Sorry, cosmologists..
    duration: Duration,
}

impl From<Duration> for RelativeDuration {
    /// Makes a new `RelativeDuration` from a `chrono::Duration`.
    #[inline]
    fn from(item: Duration) -> Self {
        RelativeDuration {
            months: 0,
            duration: item,
        }
    }
}

impl From<StdDuration> for RelativeDuration {
    /// Makes a new `RelativeDuration` from a std `Duration`.
    #[inline]
    fn from(item: StdDuration) -> Self {
        RelativeDuration::from(
            Duration::from_std(item).expect("RelativeDuration::from_std OutOfRangeError"),
        )
    }
}

impl RelativeDuration {
    /// Makes a new `RelativeDuration` with given number of years.
    ///
    /// Equivalent to `RealtiveDuration::months(years * 12)` with overflow checks.
    /// Panics when the duration is out of bounds.
    #[inline]
    pub fn years(years: i32) -> RelativeDuration {
        let months = years
            .checked_mul(12)
            .expect("RelativeDuration::years out of bounds");
        RelativeDuration::months(months)
    }

    /// Makes a new `RelativeDuration` with given number of months.
    /// Panics when the duration is out of bounds.
    #[inline]
    pub fn months(months: i32) -> RelativeDuration {
        RelativeDuration {
            months,
            duration: Duration::zero(),
        }
    }

    /// Makes a new `RelativeDuration` with given number of weeks.
    /// Panics when the duration is out of bounds.
    #[inline]
    pub fn weeks(weeks: i64) -> RelativeDuration {
        RelativeDuration {
            months: 0,
            duration: Duration::weeks(weeks),
        }
    }

    /// Makes a new `RelativeDuration` with given number of days.
    /// Panics when the duration is out of bounds.
    #[inline]
    pub fn days(days: i64) -> RelativeDuration {
        RelativeDuration {
            months: 0,
            duration: Duration::days(days),
        }
    }

    /// Makes a new `RelativeDuration` with given number of hours.
    /// Panics when the duration is out of bounds.
    #[inline]
    pub fn hours(hours: i64) -> RelativeDuration {
        RelativeDuration {
            months: 0,
            duration: Duration::hours(hours),
        }
    }

    /// Makes a new `RelativeDuration` with given number of minutes.
    /// Panics when the duration is out of bounds.
    #[inline]
    pub fn minutes(minutes: i64) -> RelativeDuration {
        RelativeDuration {
            months: 0,
            duration: Duration::minutes(minutes),
        }
    }

    /// Makes a new `RelativeDuration` with given number of seconds.
    /// Panics when the duration is out of bounds.
    #[inline]
    pub fn seconds(seconds: i64) -> RelativeDuration {
        RelativeDuration {
            months: 0,
            duration: Duration::seconds(seconds),
        }
    }

    /// Update the `Duration` part of the current `RelativeDuration`.
    #[inline]
    pub fn with_duration(self, duration: Duration) -> RelativeDuration {
        RelativeDuration {
            months: self.months,
            duration,
        }
    }
}

impl Neg for RelativeDuration {
    type Output = RelativeDuration;

    #[inline]
    fn neg(self) -> RelativeDuration {
        RelativeDuration {
            months: -self.months,
            duration: -self.duration,
        }
    }
}

impl Add<RelativeDuration> for RelativeDuration {
    type Output = RelativeDuration;

    #[inline]
    fn add(self, rhs: RelativeDuration) -> RelativeDuration {
        RelativeDuration {
            months: self.months + rhs.months,
            duration: self.duration + rhs.duration,
        }
    }
}

impl Add<Duration> for RelativeDuration {
    type Output = RelativeDuration;

    #[inline]
    fn add(self, rhs: Duration) -> RelativeDuration {
        self + RelativeDuration {
            months: 0,
            duration: rhs,
        }
    }
}

impl Add<RelativeDuration> for Duration {
    type Output = RelativeDuration;

    #[inline]
    fn add(self, rhs: RelativeDuration) -> RelativeDuration {
        rhs + self
    }
}

impl Sub for RelativeDuration {
    type Output = RelativeDuration;

    #[inline]
    fn sub(self, rhs: RelativeDuration) -> RelativeDuration {
        self + (-rhs)
    }
}

impl Sub<RelativeDuration> for Duration {
    type Output = RelativeDuration;

    #[inline]
    fn sub(self, rhs: RelativeDuration) -> RelativeDuration {
        -rhs + self
    }
}

impl Sub<Duration> for RelativeDuration {
    type Output = RelativeDuration;

    #[inline]
    fn sub(self, rhs: Duration) -> RelativeDuration {
        self + (-rhs)
    }
}

impl Mul<i32> for RelativeDuration {
    type Output = RelativeDuration;

    #[inline]
    fn mul(self, rhs: i32) -> RelativeDuration {
        RelativeDuration {
            months: self.months * rhs,
            duration: self.duration * rhs,
        }
    }
}

impl Div<i32> for RelativeDuration {
    type Output = RelativeDuration;

    #[inline]
    fn div(self, rhs: i32) -> RelativeDuration {
        RelativeDuration {
            months: self.months / rhs,
            duration: self.duration / rhs,
        }
    }
}

// The following is just copy-pasta, mostly because we
// can't impl<T> Add<RelativeDuration> for T with T: Datelike
impl Add<RelativeDuration> for NaiveDate {
    type Output = NaiveDate;

    #[inline]
    fn add(self, rhs: RelativeDuration) -> NaiveDate {
        shift_months(self, rhs.months) + rhs.duration
    }
}

impl Add<RelativeDuration> for NaiveDateTime {
    type Output = NaiveDateTime;

    #[inline]
    fn add(self, rhs: RelativeDuration) -> NaiveDateTime {
        shift_months(self, rhs.months) + rhs.duration
    }
}

impl<Tz> Add<RelativeDuration> for Date<Tz>
where
    Tz: TimeZone,
{
    type Output = Date<Tz>;

    #[inline]
    fn add(self, rhs: RelativeDuration) -> Date<Tz> {
        shift_months(self, rhs.months) + rhs.duration
    }
}

impl<Tz> Add<RelativeDuration> for DateTime<Tz>
where
    Tz: TimeZone,
{
    type Output = DateTime<Tz>;

    #[inline]
    fn add(self, rhs: RelativeDuration) -> DateTime<Tz> {
        shift_months(self, rhs.months) + rhs.duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_arithmetic() {
        let x = RelativeDuration {
            months: 5 * 12 + 7,
            duration: Duration::seconds(100),
        };
        let y = RelativeDuration {
            months: 3 * 12 + 6,
            duration: Duration::seconds(300),
        };
        let z = Duration::days(100);

        assert_eq!(
            x + y,
            RelativeDuration {
                months: 9 * 12 + 1,
                duration: Duration::seconds(400)
            }
        );
        assert_eq!(
            x - y,
            RelativeDuration {
                months: 2 * 12 + 1,
                duration: Duration::seconds(-200)
            }
        );
        assert_eq!(
            x + z,
            RelativeDuration {
                months: 5 * 12 + 7,
                duration: Duration::days(100) + Duration::seconds(100)
            }
        );

        assert_eq!(y + x, y + x, "Addition should be symmetric");
        assert_eq!(x - y, -(y - x), "Subtraction should be anti-symmetric");
        assert_eq!(y + z, z + y, "Addition should be symmetric");
        assert_eq!(y - z, -(z - y), "Subtraction should be anti-symmetric");

        assert_eq!(
            x / 2,
            RelativeDuration {
                months: 5 * 6 + 3,
                duration: Duration::seconds(50)
            }
        );
        assert_eq!(
            x * 2,
            RelativeDuration {
                months: 10 * 12 + 14,
                duration: Duration::seconds(200)
            }
        );
    }

    #[test]
    fn test_date_arithmetic() {
        let base = NaiveDate::from_ymd(2020, 2, 29);

        assert_eq!(
            base + RelativeDuration {
                months: 24,
                duration: Duration::zero()
            },
            NaiveDate::from_ymd(2022, 2, 28)
        );
        assert_eq!(
            base + RelativeDuration {
                months: 48,
                duration: Duration::zero()
            },
            NaiveDate::from_ymd(2024, 2, 29)
        );

        let not_leap = NaiveDate::from_ymd(2020, 2, 28);
        let tricky_delta = RelativeDuration {
            months: 24,
            duration: Duration::days(1),
        };
        assert_eq!(base + tricky_delta, NaiveDate::from_ymd(2022, 3, 1));
        assert_eq!(base + tricky_delta, not_leap + tricky_delta);
    }

    #[test]
    fn test_constructors() {
        assert_eq!(RelativeDuration::years(5), RelativeDuration::months(60));
        assert_eq!(RelativeDuration::weeks(5), RelativeDuration::days(35));
        assert_eq!(RelativeDuration::days(5), RelativeDuration::hours(120));
        assert_eq!(RelativeDuration::hours(5), RelativeDuration::minutes(300));
        assert_eq!(RelativeDuration::minutes(5), RelativeDuration::seconds(300));
        assert_eq!(
            RelativeDuration::months(1).with_duration(Duration::weeks(3)),
            RelativeDuration {
                months: 1,
                duration: Duration::weeks(3)
            },
        );
    }
}
