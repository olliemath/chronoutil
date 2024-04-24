//! Implements `DateRule` - an iterator yielding evenly spaced dates.
use std::iter::Iterator;

use super::delta::with_day;
use super::relative_duration::RelativeDuration;
use chrono::{Date, DateTime, Datelike, NaiveDate, NaiveDateTime, TimeZone};

/// DateRule is an iterator for yielding evenly spaced dates
/// according to a given RelativeDuration. It avoids some
/// of the pitfalls that naive usage of RelativeDuration
/// can incur.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DateRule<D: Datelike> {
    freq: RelativeDuration,
    start: D,
    end: Option<D>,
    count: Option<usize>,
    rolling_day: Option<u32>,
    _current_count: usize,
}

impl<D> DateRule<D>
where
    D: Datelike + Copy,
{
    /// Creates a new `DateRule` from an initial date and relative duration.
    #[inline]
    pub fn new(start: D, freq: RelativeDuration) -> Self {
        Self {
            freq,
            start,
            end: None,
            count: None,
            rolling_day: None,
            _current_count: 0,
        }
    }

    /// Creates a `DateRule` yielding dates one second appart.
    #[inline]
    pub fn secondly(from: D) -> Self {
        Self::new(from, RelativeDuration::seconds(1))
    }

    /// Creates a `DateRule` yielding dates one minute appart.
    #[inline]
    pub fn minutely(from: D) -> Self {
        Self::new(from, RelativeDuration::minutes(1))
    }

    /// Creates a `DateRule` yielding dates one hour appart.
    #[inline]
    pub fn hourly(from: D) -> Self {
        Self::new(from, RelativeDuration::hours(1))
    }

    /// Creates a `DateRule` yielding dates one day appart.
    #[inline]
    pub fn daily(from: D) -> Self {
        Self::new(from, RelativeDuration::days(1))
    }

    /// Creates a `DateRule` yielding dates one week appart.
    #[inline]
    pub fn weekly(from: D) -> Self {
        Self::new(from, RelativeDuration::weeks(1))
    }

    /// Creates a `DateRule` yielding dates one month appart.
    /// Ambiguous month-ends are shifted backwards as necessary.
    #[inline]
    pub fn monthly(from: D) -> Self {
        Self::new(from, RelativeDuration::months(1))
    }

    /// Creates a `DateRule` yielding dates one year appart.
    /// Ambiguous month-ends are shifted backwards as necessary.
    #[inline]
    pub fn yearly(from: D) -> Self {
        Self::new(from, RelativeDuration::years(1))
    }

    /// Limits the `DateRule` to a given number of dates.
    pub fn with_count(&self, number: usize) -> Self {
        Self {
            freq: self.freq,
            start: self.start,
            end: None,
            count: Some(number),
            rolling_day: self.rolling_day,
            _current_count: 0,
        }
    }

    /// Limits the `DateRule` to an extremal date (exclusive).
    ///
    /// If using a `RelativeDuration` which shifts dates backwards, the `end` date should
    /// be before the current date.
    ///
    /// WARNING: a forward-shifting duration with an end-date before the initial date
    /// will result in an iterator which does not terminate.
    pub fn with_end(&self, end: D) -> Self {
        Self {
            freq: self.freq,
            start: self.start,
            end: Some(end),
            count: None,
            rolling_day: self.rolling_day,
            _current_count: 0,
        }
    }

    /// Ensure the `DateRule` yields new dates which are *always* fall on the
    /// given rolling day (modulo backwards shifting for month ends). Returns
    /// Err if the rolling day is not in the range 1-31.
    ///
    /// For example:
    /// ```rust
    /// # use chrono::NaiveDate;
    /// # use chronoutil::DateRule;
    /// let start = NaiveDate::from_ymd_opt(2020, 2, 29).unwrap();
    /// let mut rule = DateRule::monthly(start).with_rolling_day(31).unwrap();
    ///
    /// assert_eq!(rule.next().unwrap(), NaiveDate::from_ymd_opt(2020, 2, 29).unwrap());
    /// assert_eq!(rule.next().unwrap(), NaiveDate::from_ymd_opt(2020, 3, 31).unwrap());
    /// assert_eq!(rule.next().unwrap(), NaiveDate::from_ymd_opt(2020, 4, 30).unwrap());
    /// assert_eq!(rule.next().unwrap(), NaiveDate::from_ymd_opt(2020, 5, 31).unwrap());
    /// // etc.
    /// ```
    ///
    /// It produces values equivalent to
    /// ```ignore
    /// rule.map(|d| with_day(d, rolling_day).unwrap())
    /// ```
    pub fn with_rolling_day(&self, rolling_day: u32) -> Result<Self, String> {
        if rolling_day == 0 || rolling_day > 31 {
            Err(format!("Rolling day {} not in range 1-31", rolling_day))
        } else {
            Ok(Self {
                freq: self.freq,
                start: self.start,
                end: self.end,
                count: self.count,
                rolling_day: Some(rolling_day),
                _current_count: self._current_count,
            })
        }
    }
}

// The following is just copy-pasta, mostly because we
// can't impl<T> Add<RelativeDuration> for T with T: Datelike
impl Iterator for DateRule<NaiveDate> {
    type Item = NaiveDate;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count.is_some() && self._current_count >= self.count.unwrap() {
            return None;
        }

        let mut current_date = self.start + self.freq * self._current_count as i32;
        if let Some(rolling_day) = self.rolling_day {
            current_date = with_day(current_date, rolling_day).unwrap();
        }

        if let Some(end) = &self.end {
            if (*end >= self.start && current_date >= *end)
                || (*end < self.start && current_date <= *end)
            {
                return None;
            }
        }

        self._current_count += 1;
        Some(current_date)
    }
}

impl Iterator for DateRule<NaiveDateTime> {
    type Item = NaiveDateTime;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count.is_some() && self._current_count >= self.count.unwrap() {
            return None;
        }

        let mut current_date = self.start + self.freq * self._current_count as i32;
        if let Some(rolling_day) = self.rolling_day {
            current_date = with_day(current_date, rolling_day).unwrap();
        }

        if let Some(end) = &self.end {
            if (*end >= self.start && current_date >= *end)
                || (*end < self.start && current_date <= *end)
            {
                return None;
            }
        }

        self._current_count += 1;
        Some(current_date)
    }
}

impl<Tz> Iterator for DateRule<Date<Tz>>
where
    Tz: TimeZone,
{
    type Item = Date<Tz>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count.is_some() && self._current_count >= self.count.unwrap() {
            return None;
        }

        let mut current_date = self.start.clone() + self.freq * self._current_count as i32;
        if let Some(rolling_day) = self.rolling_day {
            current_date = with_day(current_date, rolling_day).unwrap();
        }

        if let Some(end) = &self.end {
            if (*end >= self.start && current_date >= *end)
                || (*end < self.start && current_date <= *end)
            {
                return None;
            }
        }

        self._current_count += 1;
        Some(current_date)
    }
}

impl<Tz> Iterator for DateRule<DateTime<Tz>>
where
    Tz: TimeZone,
{
    type Item = DateTime<Tz>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.count.is_some() && self._current_count >= self.count.unwrap() {
            return None;
        }

        let mut current_date = self.start.clone() + self.freq * self._current_count as i32;
        if let Some(rolling_day) = self.rolling_day {
            current_date = with_day(current_date, rolling_day).unwrap();
        }

        if let Some(end) = &self.end {
            if (*end >= self.start && current_date >= *end)
                || (*end < self.start && current_date <= *end)
            {
                return None;
            }
        }

        self._current_count += 1;
        Some(current_date)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{Duration, NaiveDateTime, NaiveTime};

    #[test]
    fn test_rrule_with_date() {
        let start = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();

        // Seconds, hours, minutes etc
        for (i, date) in DateRule::secondly(start)
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
        for (i, date) in DateRule::minutely(start)
            .with_count(24 * 60 * 2)
            .enumerate()
        {
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
        for (i, date) in DateRule::hourly(start).with_count(24 * 2).enumerate() {
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
        let days: Vec<NaiveDate> = DateRule::daily(start).with_count(5).collect();
        assert_eq!(days[0], start, "DateRule should start at the initial day");
        assert_eq!(
            days[1],
            start + Duration::days(1),
            "DateRule should increment in days"
        );
        assert_eq!(
            days.len(),
            5,
            "DateRule should finish before the count is up"
        );

        let finish = NaiveDate::from_ymd_opt(2020, 1, 29).unwrap();
        let weeks: Vec<NaiveDate> = DateRule::weekly(start).with_end(finish).collect();
        assert_eq!(weeks[0], start, "DateRule should start at the initial day");
        assert_eq!(
            weeks[1],
            start + Duration::days(7),
            "DateRule should increment in weeks"
        );
        assert_eq!(
            weeks.len(),
            4,
            "DateRule should finish before the final day"
        );

        // Months, years
        let interesting = NaiveDate::from_ymd_opt(2020, 1, 30).unwrap(); // The day will change each month

        let months: Vec<NaiveDate> = DateRule::monthly(interesting).with_count(5).collect();
        assert_eq!(
            months[0], interesting,
            "DateRule should start at the initial day"
        );
        assert_eq!(
            months[1],
            NaiveDate::from_ymd_opt(2020, 2, 29).unwrap(),
            "DateRule should handle Feb"
        );
        assert_eq!(
            months[2],
            NaiveDate::from_ymd_opt(2020, 3, 30).unwrap(),
            "DateRule should not loose days"
        );
        assert_eq!(
            months.len(),
            5,
            "DateRule should finish before the count is up"
        );

        let years: Vec<NaiveDate> = DateRule::yearly(interesting).with_count(3).collect();
        assert_eq!(
            years[0], interesting,
            "DateRule should start at the initial day"
        );
        assert_eq!(
            years[1],
            NaiveDate::from_ymd_opt(2021, 1, 30).unwrap(),
            "DateRule should increment in years"
        );
        assert_eq!(
            years.len(),
            3,
            "DateRule should finish before the count is up"
        );
    }

    #[test]
    fn test_rrule_with_datetime() {
        // Seconds
        let o_clock = NaiveTime::from_hms_opt(1, 2, 3).unwrap();
        let day = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();
        let start = NaiveDateTime::new(day, o_clock);

        let seconds_passed = 60 * 60 + 2 * 60 + 3;

        for (i, date) in DateRule::secondly(start)
            .with_count(24 * 60 * 60 * 2)
            .enumerate()
        {
            if i > 0 {
                assert!(date > start, "Time should increase");
            }

            if i < 24 * 60 * 60 - seconds_passed {
                assert_eq!(
                    date.date(),
                    day,
                    "Expected {} seconds to be on first day",
                    date
                );
            } else if i < 2 * 24 * 60 * 60 - seconds_passed {
                assert_eq!(
                    date.date(),
                    day + Duration::days(1),
                    "Expected {} to be on second day",
                    date
                );
            } else {
                assert_eq!(
                    date.date(),
                    day + Duration::days(2),
                    "Expected {} to be on third day",
                    date
                );
            }
        }

        // Months
        let interesting = NaiveDate::from_ymd_opt(2020, 1, 30).unwrap(); // The day will change each month
        let istart = NaiveDateTime::new(interesting, o_clock);

        let months: Vec<NaiveDateTime> = DateRule::monthly(istart).with_count(5).collect();
        assert_eq!(
            months[0], istart,
            "DateRule should start at the initial day"
        );
        assert_eq!(
            months[1].date(),
            NaiveDate::from_ymd_opt(2020, 2, 29).unwrap(),
            "DateRule should handle Feb"
        );
        assert_eq!(months[1].time(), o_clock, "Time should remain the same");
        assert_eq!(
            months[2].date(),
            NaiveDate::from_ymd_opt(2020, 3, 30).unwrap(),
            "DateRule should not loose days"
        );
        assert_eq!(months[2].time(), o_clock, "Time should remain the same");
    }

    #[test]
    fn test_rrule_edge_cases() {
        let start = NaiveDate::from_ymd_opt(2020, 1, 1).unwrap();

        // Zero count
        let mut dates: Vec<NaiveDate> = DateRule::daily(start).with_count(0).collect();
        assert_eq!(dates.len(), 0);

        // End equals start
        dates = DateRule::daily(start).with_end(start).collect();
        assert_eq!(dates.len(), 0);

        // End before start
        // TODO: the only way to know to stop is to determine the forward/backwardness of the duration.
        // This is a concept which is ill formed (e.g. +1 month - 30 days) so needs thought.
        // dates = DateRule::daily(start).with_end(start - Duration::days(1)).collect();
        // assert_eq!(dates.len(), 0);
    }

    #[test]
    fn test_backwards_rrule() {
        let start = NaiveDate::from_ymd_opt(2020, 3, 31).unwrap();
        let end = NaiveDate::from_ymd_opt(2019, 12, 31).unwrap();
        let freq = RelativeDuration::months(-1);

        let dates1: Vec<NaiveDate> = DateRule::new(start, freq).with_count(3).collect();
        assert_eq!(dates1.len(), 3);
        assert_eq!(dates1[0], NaiveDate::from_ymd_opt(2020, 3, 31).unwrap());
        assert_eq!(dates1[1], NaiveDate::from_ymd_opt(2020, 2, 29).unwrap());
        assert_eq!(dates1[2], NaiveDate::from_ymd_opt(2020, 1, 31).unwrap());

        let dates2: Vec<NaiveDate> = DateRule::new(start, freq).with_end(end).collect();

        assert_eq!(dates1.len(), dates2.len());
        for k in 0..dates1.len() {
            assert_eq!(dates1[k], dates2[k]);
        }
    }

    #[test]
    fn test_long_running_rules() {
        // Sanity tests for long-running shifts with various start months
        for month in &[1, 3, 5, 7, 8, 10, 12] {
            let start = NaiveDate::from_ymd_opt(2020, *month as u32, 31).unwrap();
            let mut rule = DateRule::monthly(start);

            for _ in 0..120 {
                let shifted = rule.next().unwrap();
                if shifted.month() == 1 {
                    assert_eq!(shifted.day(), 31)
                } else if shifted.month() == 4 {
                    assert_eq!(shifted.day(), 30)
                }
            }

            let freq = RelativeDuration::months(-1);
            let mut rule = DateRule::new(start, freq);

            for _ in 0..120 {
                let shifted = rule.next().unwrap();
                if shifted.month() == 1 {
                    assert_eq!(shifted.day(), 31)
                } else if shifted.month() == 4 {
                    assert_eq!(shifted.day(), 30)
                }
            }
        }
    }
}
