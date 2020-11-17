use std::iter::Iterator;

use super::relative_duration::RelativeDuration;
use chrono::{Date, DateTime, Datelike, NaiveDate, NaiveDateTime, TimeZone};

// RRule is an iterator for yielding evenly spaced dates
// according to a given RelativeDuration. It avoids some
// of the pitfalls that naive usage of RelativeDuration
// can incur.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RRule<D: Datelike> {
    freq: RelativeDuration,
    from: D,
    until: Option<D>,
    count: Option<usize>,
    _current_count: usize,
}

impl<D> RRule<D>
where
    D: Datelike + Copy,
{
    // Create a new RRule from an initial date and relative duration.
    #[inline]
    fn new(from: D, freq: RelativeDuration) -> Self {
        Self {
            freq: freq,
            from: from,
            until: None,
            count: None,
            _current_count: 0,
        }
    }

    // Create an RRule yielding dates one second appart.
    #[inline]
    fn secondly(from: D) -> Self {
        Self::new(from, RelativeDuration::seconds(1))
    }

    // Create an RRule yielding dates one minute appart.
    #[inline]
    fn minutely(from: D) -> Self {
        Self::new(from, RelativeDuration::minutes(1))
    }

    // Create an RRule yielding dates one hour appart.
    #[inline]
    fn hourly(from: D) -> Self {
        Self::new(from, RelativeDuration::hours(1))
    }

    // Create an RRule yielding dates one day appart.
    #[inline]
    fn daily(from: D) -> Self {
        Self::new(from, RelativeDuration::days(1))
    }

    // Create an RRule yielding dates one week appart.
    #[inline]
    fn weekly(from: D) -> Self {
        Self::new(from, RelativeDuration::weeks(1))
    }

    // Create an RRule yielding dates one month appart.
    #[inline]
    fn monthly(from: D) -> Self {
        Self::new(from, RelativeDuration::months(1))
    }

    // Create an RRule yielding dates one year appart.
    #[inline]
    fn yearly(from: D) -> Self {
        Self::new(from, RelativeDuration::years(1))
    }

    // Limit the RRule to a given number of dates.
    fn with_count(&self, number: usize) -> Self {
        Self {
            freq: self.freq,
            from: self.from,
            until: None,
            count: Some(number),
            _current_count: 0,
        }
    }

    // Limit the RRule to a maximim date (exclusive).
    fn with_finish(&self, finish: D) -> Self {
        Self {
            freq: self.freq,
            from: self.from,
            until: Some(finish),
            count: None,
            _current_count: 0,
        }
    }
}

// The following is just copy-pasta, mostly because we
// can't impl<T> Add<RelativeDuration> for T with T: Datelike
impl Iterator for RRule<NaiveDate> {
    type Item = NaiveDate;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count.is_some() && self._current_count >= self.count.unwrap() {
            return None;
        }

        let current_date = self.from + self.freq * self._current_count as i32;
        if self.until.is_some() && current_date >= self.until.unwrap() {
            return None;
        }

        self._current_count += 1;
        Some(current_date)
    }
}

impl Iterator for RRule<NaiveDateTime> {
    type Item = NaiveDateTime;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count.is_some() && self._current_count >= self.count.unwrap() {
            return None;
        }

        let current_date = self.from + self.freq * self._current_count as i32;
        if self.until.is_some() && current_date >= self.until.unwrap() {
            return None;
        }

        self._current_count += 1;
        Some(current_date)
    }
}

impl<Tz> Iterator for RRule<Date<Tz>>
where
    Tz: TimeZone,
{
    type Item = Date<Tz>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count.is_some() && self._current_count >= self.count.unwrap() {
            return None;
        }

        let current_date = self.from.clone() + self.freq * self._current_count as i32;
        if self.until.is_some() && Some(current_date.clone()) >= self.until {
            return None;
        }

        self._current_count += 1;
        Some(current_date)
    }
}

impl<Tz> Iterator for RRule<DateTime<Tz>>
where
    Tz: TimeZone,
{
    type Item = DateTime<Tz>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count.is_some() && self._current_count >= self.count.unwrap() {
            return None;
        }

        let current_date = self.from.clone() + self.freq * self._current_count as i32;
        if self.until.is_some() && Some(current_date.clone()) >= self.until {
            return None;
        }

        self._current_count += 1;
        Some(current_date)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::Duration;

    #[test]
    fn test_rrule_with_date() {
        let start = NaiveDate::from_ymd(2020, 1, 1);

        // Seconds, hours, minutes etc
        for (i, date) in RRule::secondly(start)
            .with_count(24 * 60 * 60 * 2)
            .enumerate()
        {
            if i < 24 * 60 * 60 {
                assert_eq!(date, start, "Expected {} seconds to be on first day", i);
            } else {
                assert_eq!(
                    date,
                    start + Duration::days(1),
                    "Expected {} seconds to be on second day",
                    i
                );
            }
        }
        for (i, date) in RRule::minutely(start).with_count(24 * 60 * 2).enumerate() {
            if i < 24 * 60 {
                assert_eq!(date, start, "Expected {} minutes to be on first day", i);
            } else {
                assert_eq!(
                    date,
                    start + Duration::days(1),
                    "Expected {} minutes to be on second day",
                    i
                );
            }
        }
        for (i, date) in RRule::hourly(start).with_count(24 * 2).enumerate() {
            if i < 24 {
                assert_eq!(date, start, "Expected {} hours to be on first day", i);
            } else {
                assert_eq!(
                    date,
                    start + Duration::days(1),
                    "Expected {} hours to be on second day",
                    i
                );
            }
        }

        // Days, weeks
        let days: Vec<NaiveDate> = RRule::daily(start).with_count(5).collect();
        assert_eq!(days[0], start, "RRule should start at the initial day");
        assert_eq!(
            days[1],
            start + Duration::days(1),
            "RRule should increment in days"
        );
        assert_eq!(days.len(), 5, "RRule should finish before the count is up");

        let finish = NaiveDate::from_ymd(2020, 1, 29);
        let weeks: Vec<NaiveDate> = RRule::weekly(start).with_finish(finish).collect();
        assert_eq!(weeks[0], start, "RRule should start at the initial day");
        assert_eq!(
            weeks[1],
            start + Duration::days(7),
            "RRule should increment in weeks"
        );
        assert_eq!(weeks.len(), 4, "RRule should finish before the final day");

        // Months, years
        let interesting = NaiveDate::from_ymd(2020, 1, 30); // The day will change each month

        let months: Vec<NaiveDate> = RRule::monthly(interesting).with_count(5).collect();
        assert_eq!(
            months[0], interesting,
            "RRule should start at the initial day"
        );
        assert_eq!(
            months[1],
            NaiveDate::from_ymd(2020, 2, 29),
            "RRule should handle Feb"
        );
        assert_eq!(
            months[2],
            NaiveDate::from_ymd(2020, 3, 30),
            "RRule should not loose days"
        );
        assert_eq!(
            months.len(),
            5,
            "RRule should finish before the count is up"
        );

        let years: Vec<NaiveDate> = RRule::yearly(interesting).with_count(3).collect();
        assert_eq!(
            years[0], interesting,
            "RRule should start at the initial day"
        );
        assert_eq!(
            years[1],
            NaiveDate::from_ymd(2021, 1, 30),
            "RRule should increment in years"
        );
        assert_eq!(years.len(), 3, "RRule should finish before the count is up");
    }

    #[test]
    fn test_rrule_with_datetime() {
        todo!()
    }

    #[test]
    fn test_rrule_edge_cases() {
        todo!()
    }
}
