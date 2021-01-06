# [ChronoUtil][docsrs]: powerful extensions to Rust's [Chrono](https://github.com/chronotope/chrono) crate.

[![ChronoUtil GitHub Actions][gh-image]][gh-checks]
[![ChronoUtil on crates.io][cratesio-image]][cratesio]
[![ChronoUtil on docs.rs][docsrs-image]][docsrs]

[gh-image]: https://github.com/olliemath/chronoutil/workflows/test/badge.svg
[gh-checks]: https://github.com/olliemath/chronoutil/actions?query=workflow%3Atest
[cratesio-image]: https://img.shields.io/crates/v/chronoutil.svg
[cratesio]: https://crates.io/crates/chronoutil
[docsrs-image]: https://docs.rs/chronoutil/badge.svg
[docsrs]: https://docs.rs/chronoutil

ChronoUtil provides the following utilities:

- `RelativeDuration`: extending Chrono's `Duration` to add months and years
- `DateRule`s: useful iterators yielding regular (e.g. monthly) dates
- Procedural helper functions for shifting datelike values by months and years

It is heavily inspired by Python's [dateutil](https://github.com/dateutil/dateutil)
and provides a similar API, but with less of the niche functionality.

## Usage

Put this in your `Cargo.toml`:

```toml
[dependencies]
chronoutil = "0.2.0"
```

## Overview

### RelativeDuration

ChronoUtils uses a [**`RelativeDuration`**](https://docs.rs/chronoutil/0.2.0/chronoutil/relative_duration/struct.RelativeDuration.html) type to represent the magnitude of a time span
which may not be absolute (i.e. which is not simply a fixed number of nanoseconds).
A relative duration is made up of a number of months together with an absolute [`Duration`]()
component.

### DateRule

ChronoUtil provides a
[**`DateRule`**](https://docs.rs/chronoutil/0.2.0/chronoutil/rule/struct.DateRule.html)
iterator to reliably generate a collection of dates at regular intervals.
For example, the following will yield one `NaiveDate` on the last day of each
month in 2025:

```rust
let start = NaiveDate::ymd(2025, 1, 31);
let rule = DateRule<NaiveDate>::monthly(start).with_count(12);
// 2025-1-31, 2025-2-28, 2025-3-31, 2025-4-30, ...
```

### Shift functions

ChronoUtil also exposes useful shift functions which are used internally, namely:

- [**`shift_months`**](https://docs.rs/chronoutil/0.2.0/chronoutil/delta/fn.shift_months.html) to shift a datelike value by a given number of months
- [**`shift_years`**](https://docs.rs/chronoutil/0.2.0/chronoutil/delta/fn.shift_years.html) to shift a datelike value by a given number of years
- [**`with_month`**](https://docs.rs/chronoutil/0.2.0/chronoutil/delta/fn.with_month.html) to shift a datelike value to a given month
- [**`with_year`**](https://docs.rs/chronoutil/0.2.0/chronoutil/delta/fn.with_year.html) to shift a datelike value to a given year

## Design decisions and gotchas

We favour simplicity over complexity: we use only the Gregorian calendar and
make no changes e.g. for dates before the 1500s.

For days between the 1st and 28th, shifting by months has an obvious
unambiguous meaning which we always stick to. One month after Jan 28th is
always Feb 28th. Shifting Feb 28th by another month will give Mar 28th.

When shifting a day that has no equivalent in another month (e.g. asking
for one month after Jan 30th), we first compute the target month, and then if
the corresponding day does not exist in that month, we take the final day of the
month as the result. So, on a leap year, one month after Jan 30th is Feb 29th.

The order of precidence for a `RelativeDuration` is as follows:

1.  Work out the target month, if shifting by months
2.  If the initial day does not exist in that month, take the final day of the month
3.  Execute any further `Duration` shifts

So a `RelativeDuration` of 1 month and 1 day applied to Jan 31st first shifts to the
last day of Feb, and then adds a single day, giving the 1st of Mar. Applying to Jan 30th
gives the same result.

Shifted dates have no _memory_ of the date they were shifted from. Thus if we shift
Jan 31st by one month and obtain Feb 28th, a further shift of one month will be Mar 28th,
_not_ Mar 31st.

This leads us to an interesting point about the `RelativeDuration`: addition is not
_associative_:

```rust
let d1 = NaiveDate::ymd(2020, 1, 31) + RelativeDuration::months(1) + RelativeDuration::months(1);
let d2 = NaiveDate::ymd(2020, 1, 31) + (RelativeDuration::months(1) + RelativeDuration::months(1));

assert_eq!(d1, NaiveDate::ymd(2020, 3, 29));
assert_eq!(d2, NaiveDate::ymd(2020, 3, 31));
```

If you want a series of shifted dates, we advise using the `DateRule`, which takes
account of some of these subtleties
