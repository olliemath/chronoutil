use core::ops::{Add, Div, Mul, Neg, Sub};

use chrono::{Date, DateTime, Duration, NaiveDate, NaiveDateTime, TimeZone};

use super::delta::shift;

/// Relative time duration with nanosecond precision.
/// This also allows for the negative duration; see individual methods for details.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct RelativeDuration {
    years: i32, // Sorry, cosmologists..
    months: i32,
    duration: Duration,
}

impl Neg for RelativeDuration {
    type Output = RelativeDuration;

    #[inline]
    fn neg(self) -> RelativeDuration {
        RelativeDuration {
            years: -self.years,
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
            years: self.years + rhs.years,
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
            years: 0,
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
            years: self.years * rhs,
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
            years: self.years / rhs,
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
        shift(self, rhs.years, rhs.months) + rhs.duration
    }
}

impl Add<RelativeDuration> for NaiveDateTime {
    type Output = NaiveDateTime;

    #[inline]
    fn add(self, rhs: RelativeDuration) -> NaiveDateTime {
        shift(self, rhs.years, rhs.months) + rhs.duration
    }
}

impl<Tz> Add<RelativeDuration> for Date<Tz>
where
    Tz: TimeZone,
{
    type Output = Date<Tz>;

    #[inline]
    fn add(self, rhs: RelativeDuration) -> Date<Tz> {
        shift(self, rhs.years, rhs.months) + rhs.duration
    }
}

impl<Tz> Add<RelativeDuration> for DateTime<Tz>
where
    Tz: TimeZone,
{
    type Output = DateTime<Tz>;

    #[inline]
    fn add(self, rhs: RelativeDuration) -> DateTime<Tz> {
        shift(self, rhs.years, rhs.months) + rhs.duration
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duration_arithmetic() {
        let x = RelativeDuration {
            years: 5,
            months: 7,
            duration: Duration::seconds(100),
        };
        let y = RelativeDuration {
            years: 3,
            months: 6,
            duration: Duration::seconds(300),
        };
        let z = Duration::days(100);

        assert_eq!(
            x + y,
            RelativeDuration {
                years: 8,
                months: 13,
                duration: Duration::seconds(400)
            }
        );
        assert_eq!(
            x - y,
            RelativeDuration {
                years: 2,
                months: 1,
                duration: Duration::seconds(-200)
            }
        );
        assert_eq!(
            x + z,
            RelativeDuration {
                years: 5,
                months: 7,
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
                years: 2,
                months: 3,
                duration: Duration::seconds(50)
            }
        );
        assert_eq!(
            x * 2,
            RelativeDuration {
                years: 10,
                months: 14,
                duration: Duration::seconds(200)
            }
        );
    }

    #[test]
    fn test_date_arithmetic() {
        let base = NaiveDate::from_ymd(2020, 2, 29);

        assert_eq!(
            base + RelativeDuration {
                years: 1,
                months: 12,
                duration: Duration::zero()
            },
            NaiveDate::from_ymd(2022, 2, 28)
        );
        assert_eq!(
            base + RelativeDuration {
                years: 3,
                months: 12,
                duration: Duration::zero()
            },
            NaiveDate::from_ymd(2024, 2, 29)
        );

        let not_leap = NaiveDate::from_ymd(2020, 2, 28);
        let tricky_delta = RelativeDuration {
            years: 1,
            months: 12,
            duration: Duration::days(1),
        };
        assert_eq!(base + tricky_delta, NaiveDate::from_ymd(2022, 3, 1));
        assert_eq!(base + tricky_delta, not_leap + tricky_delta);
    }
}
