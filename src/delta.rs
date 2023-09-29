//! Contains utility functions for shifting Date objects.
use chrono::Datelike;

/// Returns true if the year is a leap-year, as naively defined in the Gregorian calendar.
#[inline]
pub fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

// If the day lies within the month, this function has no effect. Otherwise, it shifts
// day backwards to the final day of the month.
// XXX: No attempt is made to handle days outside the 1-31 range.
#[inline]
fn normalise_day(year: i32, month: u32, day: u32) -> u32 {
    if day <= 28 {
        day
    } else if month == 2 {
        28 + is_leap_year(year) as u32
    } else if day == 31 && (month == 4 || month == 6 || month == 9 || month == 11) {
        30
    } else {
        day
    }
}

/// Shift a date by the given number of months.
/// Ambiguous month-ends are shifted backwards as necessary.
pub fn shift_months<D: Datelike>(date: D, months: i32) -> D {
    shift_months_opt(date, months).unwrap()
}

/// Same as [`shift_months`] except fallible on unresolvable dates/times.
///
/// Returns `None` rather than panicking when shift results in an ambiguous or non-existing
/// date/time (e.g. in a DST transition).
pub fn shift_months_opt<D: Datelike>(date: D, months: i32) -> Option<D> {
    let mut year = date.year() + (date.month() as i32 + months) / 12;
    let mut month = (date.month() as i32 + months) % 12;
    let mut day = date.day();

    if month < 1 {
        year -= 1;
        month += 12;
    }

    day = normalise_day(year, month as u32, day);

    // This is slow but guaranteed to succeed (short of interger overflow)
    if day <= 28 {
        date.with_day(day)?
            .with_month(month as u32)?
            .with_year(year)
    } else {
        date.with_day(1)?
            .with_month(month as u32)?
            .with_year(year)?
            .with_day(day)
    }
}

/// Shift a date by the given number of years.
/// Ambiguous month-ends are shifted backwards as necessary.
pub fn shift_years<D: Datelike>(date: D, years: i32) -> D {
    shift_years_opt(date, years).unwrap()
}

/// Same as [`shift_years`] except fallible on unresolvable dates/times.
///
/// Returns `None` rather than panicking when shift results in an ambiguous or non-existing
/// date/time (e.g. in a DST transition).
pub fn shift_years_opt<D: Datelike>(date: D, years: i32) -> Option<D> {
    shift_months_opt(date, years * 12)
}

/// Shift the date to have the given day.  Returns None if the day is not in the range 1-31.
///
/// Ambiguous month-ends are shifted backwards as necessary.
/// For example:
/// ```rust
/// # use chrono::NaiveDate;
/// # use chronoutil::with_day;
/// let start = NaiveDate::from_ymd_opt(2020, 2, 1).unwrap();
/// assert_eq!(with_day(start, 31), Some(NaiveDate::from_ymd_opt(2020, 2, 29).unwrap()));
/// assert_eq!(with_day(start, 42), None);
/// ```
pub fn with_day<D: Datelike>(date: D, day: u32) -> Option<D> {
    if day == 0 || day > 31 {
        None
    } else {
        date.with_day(normalise_day(date.year(), date.month(), day))
    }
}

/// Shift the date to have the given month. Returns None if the month is out of range.
///
/// Ambiguous month-ends are shifted backwards as necessary.
/// For example:
/// ```rust
/// # use chrono::NaiveDate;
/// # use chronoutil::with_month;
/// let start = NaiveDate::from_ymd_opt(2020, 1, 31).unwrap();
/// assert_eq!(with_month(start, 2), Some(NaiveDate::from_ymd_opt(2020, 2, 29).unwrap()));
/// assert_eq!(with_month(start, 13), None);
/// ```
pub fn with_month<D: Datelike>(date: D, month: u32) -> Option<D> {
    if month == 0 || month > 12 {
        None
    } else {
        let delta = month as i32 - date.month() as i32;
        Some(shift_months(date, delta))
    }
}

/// Similar to [`with_month`] except _also_ fallible on unresolvable dates/times.
///
/// In addition to returning `None` when the month arg is out of range, also returns `None` rather
/// than panicking when shift results in an ambiguous or non-existing date/time (e.g. in a DST
/// transition).
pub fn with_month_opt<D: Datelike>(date: D, month: u32) -> Option<D> {
    if month == 0 || month > 12 {
        None
    } else {
        let delta = month as i32 - date.month() as i32;
        shift_months_opt(date, delta)
    }
}

/// Shift the date to have the given year.
///
/// Ambiguous month-ends are shifted backwards as necessary.
/// For example:
/// ```rust
/// # use chrono::NaiveDate;
/// # use chronoutil::with_year;
/// let start = NaiveDate::from_ymd_opt(2020, 2, 29).unwrap();
/// assert_eq!(with_year(start, 2021), NaiveDate::from_ymd_opt(2021, 2, 28).unwrap());
/// ```
pub fn with_year<D: Datelike>(date: D, year: i32) -> D {
    with_year_opt(date, year).unwrap()
}

/// Same as [`with_year`] except fallible on unresolvable dates/times.
///
/// Returns `None` rather than panicking when shift results in an ambiguous or non-existing
/// date/time (e.g. in a DST transition).
pub fn with_year_opt<D: Datelike>(date: D, year: i32) -> Option<D> {
    let delta = year - date.year();
    shift_years_opt(date, delta)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use chrono::{
        naive::{NaiveDate, NaiveDateTime, NaiveTime},
        LocalResult, TimeZone,
    };

    use super::*;

    #[test]
    fn test_leap_year_cases() {
        let _leap_years: Vec<i32> = vec![
            1904, 1908, 1912, 1916, 1920, 1924, 1928, 1932, 1936, 1940, 1944, 1948, 1952, 1956,
            1960, 1964, 1968, 1972, 1976, 1980, 1984, 1988, 1992, 1996, 2000, 2004, 2008, 2012,
            2016, 2020,
        ];
        let leap_years_1900_to_2020: HashSet<i32> = _leap_years.into_iter().collect();

        for year in 1900..2021 {
            assert_eq!(is_leap_year(year), leap_years_1900_to_2020.contains(&year))
        }
    }

    #[test]
    fn test_shift_months() {
        let base = NaiveDate::from_ymd_opt(2020, 1, 31).unwrap();

        assert_eq!(
            shift_months(base, 0),
            NaiveDate::from_ymd_opt(2020, 1, 31).unwrap()
        );
        assert_eq!(
            shift_months(base, 1),
            NaiveDate::from_ymd_opt(2020, 2, 29).unwrap()
        );
        assert_eq!(
            shift_months(base, 2),
            NaiveDate::from_ymd_opt(2020, 3, 31).unwrap()
        );
        assert_eq!(
            shift_months(base, 3),
            NaiveDate::from_ymd_opt(2020, 4, 30).unwrap()
        );
        assert_eq!(
            shift_months(base, 4),
            NaiveDate::from_ymd_opt(2020, 5, 31).unwrap()
        );
        assert_eq!(
            shift_months(base, 5),
            NaiveDate::from_ymd_opt(2020, 6, 30).unwrap()
        );
        assert_eq!(
            shift_months(base, 6),
            NaiveDate::from_ymd_opt(2020, 7, 31).unwrap()
        );
        assert_eq!(
            shift_months(base, 7),
            NaiveDate::from_ymd_opt(2020, 8, 31).unwrap()
        );
        assert_eq!(
            shift_months(base, 8),
            NaiveDate::from_ymd_opt(2020, 9, 30).unwrap()
        );
        assert_eq!(
            shift_months(base, 9),
            NaiveDate::from_ymd_opt(2020, 10, 31).unwrap()
        );
        assert_eq!(
            shift_months(base, 10),
            NaiveDate::from_ymd_opt(2020, 11, 30).unwrap()
        );
        assert_eq!(
            shift_months(base, 11),
            NaiveDate::from_ymd_opt(2020, 12, 31).unwrap()
        );
        assert_eq!(
            shift_months(base, 12),
            NaiveDate::from_ymd_opt(2021, 1, 31).unwrap()
        );
        assert_eq!(
            shift_months(base, 13),
            NaiveDate::from_ymd_opt(2021, 2, 28).unwrap()
        );

        assert_eq!(
            shift_months(base, -1),
            NaiveDate::from_ymd_opt(2019, 12, 31).unwrap()
        );
        assert_eq!(
            shift_months(base, -2),
            NaiveDate::from_ymd_opt(2019, 11, 30).unwrap()
        );
        assert_eq!(
            shift_months(base, -3),
            NaiveDate::from_ymd_opt(2019, 10, 31).unwrap()
        );
        assert_eq!(
            shift_months(base, -4),
            NaiveDate::from_ymd_opt(2019, 9, 30).unwrap()
        );
        assert_eq!(
            shift_months(base, -5),
            NaiveDate::from_ymd_opt(2019, 8, 31).unwrap()
        );
        assert_eq!(
            shift_months(base, -6),
            NaiveDate::from_ymd_opt(2019, 7, 31).unwrap()
        );
        assert_eq!(
            shift_months(base, -7),
            NaiveDate::from_ymd_opt(2019, 6, 30).unwrap()
        );
        assert_eq!(
            shift_months(base, -8),
            NaiveDate::from_ymd_opt(2019, 5, 31).unwrap()
        );
        assert_eq!(
            shift_months(base, -9),
            NaiveDate::from_ymd_opt(2019, 4, 30).unwrap()
        );
        assert_eq!(
            shift_months(base, -10),
            NaiveDate::from_ymd_opt(2019, 3, 31).unwrap()
        );
        assert_eq!(
            shift_months(base, -11),
            NaiveDate::from_ymd_opt(2019, 2, 28).unwrap()
        );
        assert_eq!(
            shift_months(base, -12),
            NaiveDate::from_ymd_opt(2019, 1, 31).unwrap()
        );
        assert_eq!(
            shift_months(base, -13),
            NaiveDate::from_ymd_opt(2018, 12, 31).unwrap()
        );

        assert_eq!(
            shift_months(base, 1265),
            NaiveDate::from_ymd_opt(2125, 6, 30).unwrap()
        );
    }

    #[test]
    fn test_shift_months_with_overflow() {
        let base = NaiveDate::from_ymd_opt(2020, 12, 31).unwrap();

        assert_eq!(shift_months(base, 0), base);
        assert_eq!(
            shift_months(base, 1),
            NaiveDate::from_ymd_opt(2021, 1, 31).unwrap()
        );
        assert_eq!(
            shift_months(base, 2),
            NaiveDate::from_ymd_opt(2021, 2, 28).unwrap()
        );
        assert_eq!(
            shift_months(base, 12),
            NaiveDate::from_ymd_opt(2021, 12, 31).unwrap()
        );
        assert_eq!(
            shift_months(base, 18),
            NaiveDate::from_ymd_opt(2022, 6, 30).unwrap()
        );

        assert_eq!(
            shift_months(base, -1),
            NaiveDate::from_ymd_opt(2020, 11, 30).unwrap()
        );
        assert_eq!(
            shift_months(base, -2),
            NaiveDate::from_ymd_opt(2020, 10, 31).unwrap()
        );
        assert_eq!(
            shift_months(base, -10),
            NaiveDate::from_ymd_opt(2020, 2, 29).unwrap()
        );
        assert_eq!(
            shift_months(base, -12),
            NaiveDate::from_ymd_opt(2019, 12, 31).unwrap()
        );
        assert_eq!(
            shift_months(base, -18),
            NaiveDate::from_ymd_opt(2019, 6, 30).unwrap()
        );
    }

    #[test]
    fn test_shift_months_datetime() {
        let date = NaiveDate::from_ymd_opt(2020, 1, 31).unwrap();
        let o_clock = NaiveTime::from_hms_opt(1, 2, 3).unwrap();

        let base = NaiveDateTime::new(date, o_clock);

        assert_eq!(
            shift_months(base, 0).date(),
            NaiveDate::from_ymd_opt(2020, 1, 31).unwrap()
        );
        assert_eq!(
            shift_months(base, 1).date(),
            NaiveDate::from_ymd_opt(2020, 2, 29).unwrap()
        );
        assert_eq!(
            shift_months(base, 2).date(),
            NaiveDate::from_ymd_opt(2020, 3, 31).unwrap()
        );
        assert_eq!(shift_months(base, 0).time(), o_clock);
        assert_eq!(shift_months(base, 1).time(), o_clock);
        assert_eq!(shift_months(base, 2).time(), o_clock);
    }

    #[test]
    fn test_shift_months_datetime_tz() {
        let tz = &chrono_tz::Australia::Melbourne;

        let base = tz.with_ymd_and_hms(2020, 1, 31, 1, 2, 3).single().unwrap();

        assert_eq!(
            shift_months(base, 0),
            tz.with_ymd_and_hms(2020, 1, 31, 1, 2, 3).single().unwrap()
        );
        assert_eq!(
            shift_months(base, 1),
            tz.with_ymd_and_hms(2020, 2, 29, 1, 2, 3).single().unwrap()
        );
        assert_eq!(
            shift_months(base, 2),
            tz.with_ymd_and_hms(2020, 3, 31, 1, 2, 3).single().unwrap()
        );

        assert_eq!(
            shift_months_opt(base, 0).unwrap(),
            tz.with_ymd_and_hms(2020, 1, 31, 1, 2, 3).single().unwrap()
        );
        assert_eq!(
            shift_months_opt(base, 1).unwrap(),
            tz.with_ymd_and_hms(2020, 2, 29, 1, 2, 3).single().unwrap()
        );
        assert_eq!(
            shift_months_opt(base, 2).unwrap(),
            tz.with_ymd_and_hms(2020, 3, 31, 1, 2, 3).single().unwrap()
        );
    }

    #[test]
    #[should_panic]
    fn test_shift_months_datetime_to_dst_backward_transition() {
        let dst_tz = &chrono_tz::Australia::Melbourne;

        // On Apr 5th 2020 after 02:59:59, clocks were wound back to 02:00:00 making 02:00::00 to
        // 02:59:59 ambiguous.
        // <https://www.timeanddate.com/time/change/australia/melbourne?year=2020>
        if let LocalResult::Single(base) = dst_tz.with_ymd_and_hms(2020, 3, 5, 2, 00, 0) {
            shift_months(base, 1); // panics
        }
    }

    #[test]
    fn test_shift_months_opt_datetime_to_dst_backward_transition() {
        let dst_tz = &chrono_tz::Australia::Melbourne;

        let base = dst_tz
            .with_ymd_and_hms(2020, 3, 5, 2, 00, 0)
            .single()
            .unwrap();
        assert_eq!(None, shift_months_opt(base, 1))
    }

    #[test]
    #[should_panic]
    fn test_shift_months_datetime_to_dst_forward_transition() {
        let dst_tz = &chrono_tz::Australia::Melbourne;

        // On Oct 4th 2020 after 01:59:59, clocks were advanced to 03:00:00 making 02:00:00 to
        // 02:59:59 non-existent.
        // <https://www.timeanddate.com/time/change/australia/melbourne?year=2020>
        if let LocalResult::Single(base) = dst_tz.with_ymd_and_hms(2020, 9, 4, 2, 00, 0) {
            shift_months(base, 1); // panics
        }
    }

    #[test]
    fn test_shift_months_opt_datetime_to_dst_forward_transition() {
        let dst_tz = &chrono_tz::Australia::Melbourne;

        let base = dst_tz
            .with_ymd_and_hms(2020, 9, 4, 2, 00, 0)
            .single()
            .unwrap();
        assert_eq!(None, shift_months_opt(base, 1))
    }

    #[test]
    fn test_shift_years() {
        let base = NaiveDate::from_ymd_opt(2020, 2, 29).unwrap();

        assert_eq!(
            shift_years(base, 0),
            NaiveDate::from_ymd_opt(2020, 2, 29).unwrap()
        );
        assert_eq!(
            shift_years(base, 1),
            NaiveDate::from_ymd_opt(2021, 2, 28).unwrap()
        );
        assert_eq!(
            shift_years(base, 4),
            NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()
        );
        assert_eq!(
            shift_years(base, 80),
            NaiveDate::from_ymd_opt(2100, 2, 28).unwrap()
        );
        assert_eq!(
            shift_years(base, -1),
            NaiveDate::from_ymd_opt(2019, 2, 28).unwrap()
        );
        assert_eq!(
            shift_years(base, -4),
            NaiveDate::from_ymd_opt(2016, 2, 29).unwrap()
        );
        assert_eq!(
            shift_years(base, -20),
            NaiveDate::from_ymd_opt(2000, 2, 29).unwrap()
        );
        assert_eq!(
            shift_years(base, -120),
            NaiveDate::from_ymd_opt(1900, 2, 28).unwrap()
        );
    }

    #[test]
    fn test_with_month() {
        let base = NaiveDate::from_ymd_opt(2020, 1, 31).unwrap();

        assert_eq!(with_month(base, 0), None);
        assert_eq!(with_month(base, 1), Some(base));
        assert_eq!(
            with_month(base, 2).unwrap(),
            NaiveDate::from_ymd_opt(2020, 2, 29).unwrap()
        );
        assert_eq!(
            with_month(base, 3).unwrap(),
            NaiveDate::from_ymd_opt(2020, 3, 31).unwrap()
        );
        assert_eq!(
            with_month(base, 4).unwrap(),
            NaiveDate::from_ymd_opt(2020, 4, 30).unwrap()
        );
        assert_eq!(
            with_month(base, 5).unwrap(),
            NaiveDate::from_ymd_opt(2020, 5, 31).unwrap()
        );
        assert_eq!(
            with_month(base, 6).unwrap(),
            NaiveDate::from_ymd_opt(2020, 6, 30).unwrap()
        );
        assert_eq!(
            with_month(base, 7).unwrap(),
            NaiveDate::from_ymd_opt(2020, 7, 31).unwrap()
        );
        assert_eq!(
            with_month(base, 8).unwrap(),
            NaiveDate::from_ymd_opt(2020, 8, 31).unwrap()
        );
        assert_eq!(
            with_month(base, 9).unwrap(),
            NaiveDate::from_ymd_opt(2020, 9, 30).unwrap()
        );
        assert_eq!(
            with_month(base, 10).unwrap(),
            NaiveDate::from_ymd_opt(2020, 10, 31).unwrap()
        );
        assert_eq!(
            with_month(base, 11).unwrap(),
            NaiveDate::from_ymd_opt(2020, 11, 30).unwrap()
        );
        assert_eq!(
            with_month(base, 12).unwrap(),
            NaiveDate::from_ymd_opt(2020, 12, 31).unwrap()
        );
        assert_eq!(with_month(base, 13), None);

        assert_eq!(
            with_month(NaiveDate::from_ymd_opt(2021, 1, 31).unwrap(), 2),
            Some(NaiveDate::from_ymd_opt(2021, 2, 28).unwrap())
        );

        // Backwards shifts work too
        assert_eq!(
            with_month(NaiveDate::from_ymd_opt(2021, 2, 15).unwrap(), 1),
            Some(NaiveDate::from_ymd_opt(2021, 1, 15).unwrap())
        );
    }

    #[test]
    fn test_with_month_opt() {
        let tz = &chrono_tz::Australia::Melbourne;

        let base = tz.with_ymd_and_hms(2020, 1, 31, 0, 0, 0).single().unwrap();

        assert_eq!(with_month(base, 0), None);
        assert_eq!(with_month(base, 1), Some(base));
        assert_eq!(
            with_month(base, 2).unwrap(),
            tz.with_ymd_and_hms(2020, 2, 29, 0, 0, 0).single().unwrap()
        );
        assert_eq!(
            with_month(base, 3).unwrap(),
            tz.with_ymd_and_hms(2020, 3, 31, 0, 0, 0).single().unwrap()
        );
        assert_eq!(
            with_month(base, 12).unwrap(),
            tz.with_ymd_and_hms(2020, 12, 31, 0, 0, 0).single().unwrap()
        );
        assert_eq!(with_month(base, 13), None);

        // Backwards shifts work too
        assert_eq!(
            with_month(
                tz.with_ymd_and_hms(2021, 2, 15, 0, 0, 0).single().unwrap(),
                1
            ),
            Some(tz.with_ymd_and_hms(2021, 1, 15, 0, 0, 0).single().unwrap())
        );
    }

    #[test]
    fn test_with_year() {
        let base = NaiveDate::from_ymd_opt(2020, 2, 29).unwrap();

        assert_eq!(
            with_year(base, 2024),
            NaiveDate::from_ymd_opt(2024, 2, 29).unwrap()
        );
        assert_eq!(
            with_year(base, 2021),
            NaiveDate::from_ymd_opt(2021, 2, 28).unwrap()
        );
        assert_eq!(
            with_year(base, 2020),
            NaiveDate::from_ymd_opt(2020, 2, 29).unwrap()
        );
        assert_eq!(
            with_year(base, 2019),
            NaiveDate::from_ymd_opt(2019, 2, 28).unwrap()
        );
        assert_eq!(
            with_year(base, 2016),
            NaiveDate::from_ymd_opt(2016, 2, 29).unwrap()
        );
    }
}
