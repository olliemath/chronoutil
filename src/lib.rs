#![warn(missing_docs)]
//! # Chronoutil provides powerful extensions to rust's Chrono crate
//!
//! ChronoUtil provides the following utilities:
//! - `RelativeDuration`: extending Chrono's `Duration` to add months and years
//! - `DateRule`: useful iterators yielding regular (e.g. monthly) dates
//! - Procedural helper functions for shifting datelike values by months and years
//!
//! It is heavily inspired by Python's [dateutil](https://github.com/dateutil/dateutil)
//! and provides a similar API, but with less of the niche functionality.
//! ## Overview
//!
//! ### RelativeDuration
//!
//! ChronoUtils uses a [RelativeDuration] type to represent the magnitude of a time span
//! which may not be absolute (i.e. which is not simply a fixed number of nanoseconds).
//! A relative duration is made up of a number of months together with an absolute duration
//! component.
//!
//! ```rust
//! # use chrono::{NaiveDate};
//! # use chronoutil::{RelativeDuration};
//! let one_day = RelativeDuration::days(1);
//! let one_month = RelativeDuration::months(1);
//! let delta = one_month + one_day;
//! let start = NaiveDate::from_ymd(2020, 1, 1);
//! assert_eq!(start + delta, NaiveDate::from_ymd(2020, 2, 2));
//! ```
//!
//! The behaviour of `RelativeDuration` is consistent and well-defined in edge-cases
//! (see the Design Decisions section for an explanation):
//!
//! ```rust
//! # use chrono::{NaiveDate};
//! # use chronoutil::{RelativeDuration};
//! let one_day = RelativeDuration::days(1);
//! let one_month = RelativeDuration::months(1);
//! let delta = one_month + one_day;
//! let start = NaiveDate::from_ymd(2020, 1, 30);
//! assert_eq!(start + delta, NaiveDate::from_ymd(2020, 3, 1));
//! ```
//!
//! ### DateRule
//!
//! ChronoUtil provides a
//! [DateRule]
//! iterator to reliably generate a collection of dates at regular intervals.
//! For example, the following will yield one `NaiveDate` on the last day of each
//! month in 2025:
//!
//! ```rust
//! # use chrono::NaiveDate;
//! # use chronoutil::DateRule;
//! let start = NaiveDate::from_ymd(2025, 1, 31);
//! let rule = DateRule::monthly(start).with_count(12);
//! // 2025-1-31, 2025-2-28, 2025-3-31, 2025-4-30, ...
//! ```
//!
//! ### Shift functions
//!
//! ChronoUtil also exposes useful shift functions which are used internally, namely:
//!
//! - [shift_months] to shift a datelike value by a given number of months
//! - [shift_years] to shift a datelike value by a given number of years
//! - [with_day] to shift a datelike value to a given day
//! - [with_month] to shift a datelike value to a given month
//! - [with_year] to shift a datelike value to a given year
//!
//! ## Design decisions and gotchas
//!
//! We favour simplicity over complexity: we use only the Gregorian calendar and
//! make no changes e.g. for dates before the 1500s.
//!
//! For days between the 1st and 28th, shifting by months has an obvious
//! unambiguous meaning which we always stick to. One month after Jan 28th is
//! always Feb 28th. Shifting Feb 28th by another month will give Mar 28th.
//!
//! When shifting a day that has no equivalent in another month (e.g. asking
//! for one month after Jan 30th), we first compute the target month, and then if
//! the corresponding day does not exist in that month, we take the final day of the
//! month as the result. So, on a leap year, one month after Jan 30th is Feb 29th.
//!
//! The order of precidence for a `RelativeDuration` is as follows:
//!
//! 1.  Work out the target month, if shifting by months
//! 2.  If the initial day does not exist in that month, take the final day of the month
//! 3.  Execute any further `Duration` shifts
//!
//! So a `RelativeDuration` of 1 month and 1 day applied to Jan 31st first shifts to the
//! last day of Feb, and then adds a single day, giving the 1st of Mar. Applying to Jan 30th
//! gives the same result.
//!
//! Shifted dates have no _memory_ of the date they were shifted from. Thus if we shift
//! Jan 31st by one month and obtain Feb 28th, a further shift of one month will be Mar 28th,
//! _not_ Mar 31st.
//!
//! This leads us to an interesting point about the `RelativeDuration`: addition is not
//! _associative_:
//!
//! ```rust
//! # use chrono::NaiveDate;
//! # use chronoutil::RelativeDuration;
//!
//! let d1 = (NaiveDate::from_ymd(2020, 1, 31) + RelativeDuration::months(1))
//!             + RelativeDuration::months(1);
//! let d2 = NaiveDate::from_ymd(2020, 1, 31)
//!             + (RelativeDuration::months(1) + RelativeDuration::months(1));
//!
//! assert_eq!(d1, NaiveDate::from_ymd(2020, 3, 29));
//! assert_eq!(d2, NaiveDate::from_ymd(2020, 3, 31));
//! ```
//!
//! If you want a series of shifted dates, we advise using the `DateRule`, which takes
//! account of some of these subtleties:
//! ```rust
//! # use chrono::NaiveDate;
//! # use chronoutil::{RelativeDuration, DateRule};
//! let start = NaiveDate::from_ymd(2020, 1, 31);
//! let delta = RelativeDuration::months(1);
//! let mut rule = DateRule::new(start, delta);
//! assert_eq!(rule.next().unwrap(), NaiveDate::from_ymd(2020, 1, 31));
//! assert_eq!(rule.next().unwrap(), NaiveDate::from_ymd(2020, 2, 29));
//! assert_eq!(rule.next().unwrap(), NaiveDate::from_ymd(2020, 3, 31));
//! ```

extern crate chrono;

pub mod delta;
pub mod relative_duration;
pub mod rule;

pub use relative_duration::RelativeDuration;
pub use rule::DateRule;
// Utility functions may be useful for others
pub use delta::{is_leap_year, shift_months, shift_years, with_day, with_month, with_year};
