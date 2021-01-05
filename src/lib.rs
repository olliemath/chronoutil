#![warn(missing_docs)]
//! Chronoutil provides powerful extensions to rust's Chrono crate

extern crate chrono;

pub mod delta;
pub mod relative_duration;
pub mod rule;

pub use relative_duration::RelativeDuration;
pub use rule::DateRule;
// Utility functions may be useful for others
pub use delta::{is_leap_year, shift_months, shift_years, with_month, with_year};
