use super::RelativeDuration;
use chrono::Duration;
use std::{convert::TryInto, fmt::Write};

fn dhmsn_to_duration(
    days: i64,
    hours: i64,
    minutes: i64,
    seconds: i64,
    nanos: u32,
) -> Option<Duration> {
    Duration::new(
        days.checked_mul(24)?
            .checked_add(hours)?
            .checked_mul(60)?
            .checked_add(minutes)?
            .checked_mul(60)?
            .checked_add(seconds)?,
        nanos,
    )
}

fn get_terminated<T: std::str::FromStr + From<i32>>(
    input: &str,
    terminator: char,
) -> Result<(&str, T), String> {
    if let Some((int_string, remainder)) = input.split_once(terminator) {
        let int = int_string.parse::<T>().map_err(|_| {
            format!(
                "{} is not a valid {}",
                int_string,
                std::any::type_name::<T>()
            )
        })?;
        Ok((remainder, int))
    } else {
        Ok((input, 0.into()))
    }
}

fn get_terminated_decimal(input: &str, terminator: char) -> Result<(&str, i64, u32), String> {
    if let Some((decimal_string, remainder)) = input.split_once(terminator) {
        let (int_string, fraction_string) = decimal_string.split_once('.').unwrap_or_else(|| {
            decimal_string
                // if no '.' was found, look for ',', as both are valid decimal separators in iso 8601
                .split_once(',')
                // if neither is found take the whole string as the integer part, with no fraction
                .unwrap_or((decimal_string, ""))
        });

        let int = int_string
            .parse::<i64>()
            .map_err(|_| format!("{} is not a valid i64", int_string))?;

        let fraction = if fraction_string.is_empty() {
            0
        } else {
            fraction_string
                .chars()
                // right pad with zeros
                .chain(std::iter::repeat('0'))
                // truncate to 9 chars, since we only support nanosecond resolution
                .take(9)
                .collect::<String>()
                .parse::<u32>()
                .map_err(|_| format!("{} is not a valid u32", fraction_string))?
        };

        // special handling for case of nonzero nanoseconds on a negative duration
        if decimal_string.starts_with('-') && fraction != 0 {
            Ok((
                remainder,
                int - 1,
                (-(fraction as i32) + 1_000_000_000).try_into().unwrap(),
            ))
        } else {
            Ok((remainder, int, fraction))
        }
    } else {
        Ok((input, 0, 0))
    }
}

fn parse_datespec(datespec: &str) -> Result<(i32, i32, i64), String> {
    let (remainder, years) = get_terminated::<i32>(datespec, 'Y')?;
    let (remainder, months) = get_terminated::<i32>(remainder, 'M')?;
    let (remainder, weeks) = get_terminated::<i64>(remainder, 'W')?;
    let (remainder, days) = get_terminated::<i64>(remainder, 'D')?;

    if !remainder.is_empty() {
        Err(format!(
            "trailing characters: {} in datespec: {}",
            remainder, datespec
        ))
    } else {
        Ok((
            years,
            months,
            weeks
                .checked_mul(7)
                .and_then(|x| x.checked_add(days))
                .ok_or_else(|| "integer overflow on constructing duration".to_string())?,
        ))
    }
}

fn parse_timespec(timespec: &str) -> Result<(i64, i64, i64, u32), String> {
    let (remainder, hours) = get_terminated::<i64>(timespec, 'H')?;
    let (remainder, mins) = get_terminated::<i64>(remainder, 'M')?;
    let (remainder, secs, nanos) = get_terminated_decimal(remainder, 'S')?;

    if !remainder.is_empty() {
        Err(format!(
            "trailing characters: {} in timespec: {}",
            remainder, timespec
        ))
    } else {
        Ok((hours, mins, secs, nanos))
    }
}

impl RelativeDuration {
    /// Parses an [ISO 8601 duration string](https://en.wikipedia.org/wiki/ISO_8601#Durations) into
    /// a [`RelativeDuration`] value.
    ///
    /// This supports only duration strings with integer values (i.e `"P1Y"` but not `"P0.5Y"` or
    /// `"P0,5Y"`), as fractional values cannot be unambiguously represented as a
    /// [`RelativeDuration`]. The one exception to this is the seconds field, where the fractional
    /// part is truncated to 9 digits, and parsed as nanoseconds.
    ///
    /// # Errors
    ///
    /// - Invalid duration string input
    /// - Fractional values (apart from seconds) in duration string
    ///
    /// # Example
    ///
    /// ```
    /// use chronoutil::RelativeDuration;
    ///
    /// assert_eq!(
    ///     RelativeDuration::parse_from_iso8601("P1Y").unwrap(),
    ///     RelativeDuration::years(1),
    /// );
    /// ```
    pub fn parse_from_iso8601(input: &str) -> Result<RelativeDuration, String> {
        let input = input
            .strip_prefix('P')
            .ok_or_else(|| "duration was not prefixed with P".to_string())?;

        let (datespec, timespec) = input.split_once('T').unwrap_or((input, ""));

        let (years, months, days) = parse_datespec(datespec)?;
        let (hours, mins, secs, nanos) = parse_timespec(timespec)?;

        Ok(RelativeDuration::months(
            years
                .checked_mul(12)
                .and_then(|x| x.checked_add(months))
                .ok_or_else(|| "integer overflow on constructing duration".to_string())?,
        )
        .with_duration(
            dhmsn_to_duration(days, hours, mins, secs, nanos)
                .ok_or_else(|| "integer overflow on constructing duration".to_string())?,
        ))
    }

    /// Formats a [`RelativeDuration`] value into an
    /// [ISO 8601 duration string](https://en.wikipedia.org/wiki/ISO_8601#Durations).
    ///
    /// # Example
    ///
    /// ```
    /// use chronoutil::RelativeDuration;
    ///
    /// assert_eq!(
    ///     RelativeDuration::years(1).format_to_iso8601(),
    ///     "P1Y",
    /// );
    /// ```
    pub fn format_to_iso8601(&self) -> String {
        let years = self.months as i64 / 12;
        let months = self.months as i64 % 12;

        let duration_seconds = self.duration.num_seconds();

        let days = duration_seconds / (24 * 60 * 60);
        let mut remaining_seconds = duration_seconds % (24 * 60 * 60);

        let hours = remaining_seconds / (60 * 60);
        remaining_seconds %= 60 * 60;

        let minutes = remaining_seconds / 60;
        remaining_seconds %= 60;

        let subsec_nanos = self.duration.subsec_nanos();

        // This awkward handling is needed to represent nanoseconds as a fraction of seconds,
        // instead of independently, since it must have no sign, and will affect the sign for
        // seconds. This would be simpler if we could get the Duration.secs and Duration.nanos
        // directly, but unfortunately chrono only offers Duration::num_seconds and
        // Duration::subsec_nanos, both of which apply transformations before returning...
        let (seconds, nanos, push_minus) = if remaining_seconds > 0 && subsec_nanos < 0 {
            (remaining_seconds - 1, subsec_nanos + 1_000_000_000, false)
        } else if remaining_seconds < 0 && subsec_nanos > 0 {
            (remaining_seconds + 1, -subsec_nanos + 1_000_000_000, false)
        } else if remaining_seconds <= 0 && subsec_nanos < 0 {
            (remaining_seconds, -subsec_nanos, remaining_seconds == 0)
        } else {
            (remaining_seconds, subsec_nanos, false)
        };

        let mut out = String::new();

        out.push('P');

        [years, months, days]
            .iter()
            .zip(['Y', 'M', 'D'])
            .filter(|x| *x.0 != 0)
            .fold(&mut out, |out, x| {
                let _ = write!(out, "{}{}", x.0, x.1);
                out
            });

        if [hours, minutes, seconds, nanos as i64]
            .iter()
            .any(|x| *x != 0)
        {
            out.push('T');
        }

        [hours, minutes]
            .iter()
            .zip(['H', 'M'])
            .filter(|x| *x.0 != 0)
            .fold(&mut out, |out, x| {
                let _ = write!(out, "{}{}", x.0, x.1);
                out
            });

        if push_minus {
            out.push('-');
        }

        if seconds != 0 || nanos != 0 {
            let _ = write!(out, "{}", seconds);

            if nanos != 0 {
                let nanos_str_raw = format!("{:0>9}", nanos);
                let nanos_str_trimmed = nanos_str_raw.trim_end_matches('0');
                out.push('.');
                out.push_str(nanos_str_trimmed);
            }

            out.push('S');
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn test_parse_duration() {
        [
            (
                "P1YT1S",
                RelativeDuration::months(12).with_duration(Duration::seconds(1)),
            ),
            (
                "P2Y2M2DT2H2M2S",
                RelativeDuration::months(2 * 12 + 2)
                    .with_duration(dhmsn_to_duration(2, 2, 2, 2, 0).unwrap()),
            ),
            (
                "P1M",
                RelativeDuration::months(1).with_duration(Duration::zero()),
            ),
            ("PT10M", RelativeDuration::minutes(10)),
            ("P-1M", RelativeDuration::months(-1)),
            ("P1W1D", RelativeDuration::days(8)),
            (
                "P1Y-10M-1W3DT3H-6M-1S",
                RelativeDuration::months(2)
                    .with_duration(dhmsn_to_duration(-4, 3, -6, -1, 0).unwrap()),
            ),
            ("P-23M", RelativeDuration::months(-23)),
            ("PT0.0000000010S", RelativeDuration::nanoseconds(1)),
            ("PT0.1S", RelativeDuration::nanoseconds(100_000_000)),
            (
                "PT-0.999999999S",
                RelativeDuration::years(0)
                    .with_duration(dhmsn_to_duration(0, 0, 0, -1, 1).unwrap()),
            ),
        ]
        .iter()
        .for_each(|(input, expected)| {
            assert_eq!(
                RelativeDuration::parse_from_iso8601(input).unwrap(),
                *expected
            )
        })
    }

    #[test]
    fn test_format_duration() {
        [
            (
                RelativeDuration::months(12).with_duration(Duration::seconds(1)),
                "P1YT1S",
            ),
            (
                RelativeDuration::months(2 * 12 + 2)
                    .with_duration(dhmsn_to_duration(2, 2, 2, 2, 0).unwrap()),
                "P2Y2M2DT2H2M2S",
            ),
            (
                RelativeDuration::months(1).with_duration(Duration::zero()),
                "P1M",
            ),
            (RelativeDuration::minutes(10), "PT10M"),
            (RelativeDuration::months(-1), "P-1M"),
            (RelativeDuration::months(-23), "P-1Y-11M"),
            (RelativeDuration::nanoseconds(1), "PT0.000000001S"),
            (RelativeDuration::nanoseconds(100_000_000), "PT0.1S"),
            (
                RelativeDuration::years(0)
                    .with_duration(dhmsn_to_duration(0, 0, 0, -1, 1).unwrap()),
                "PT-0.999999999S",
            ),
        ]
        .iter()
        .for_each(|(input, expected)| assert_eq!(input.format_to_iso8601(), *expected))
    }

    proptest! {
        #[test]
        fn proptest_format_and_back(
            months in prop::num::i32::ANY,
            secs in (i64::MIN/1000)..(i64::MAX/1000),
            nanos in 0u32..1_000_000_000
        ) {
            let d = RelativeDuration::months(months).with_duration(Duration::new(secs, nanos).unwrap());
            prop_assert_eq!(d, RelativeDuration::parse_from_iso8601(&(d.format_to_iso8601())).unwrap());
        }

        #[test]
        fn proptest_parse_and_back(
            s in r"P(?:[1-9][0-9]{0,7}Y)?(?:(?:[1-9]|1[0-1])M)?(?:(?:[1-9]|[1-2][0-9])D)?(?:T(?:(?:[1-9]|1[0-9]|2[0-3])H)(?:(?:[1-9]|[1-5][0-9])M)(?:(?:(?:[1-9]|[1-5][0-9])|(?:(?:[0-9]|[1-5][0-9])\.[0-9]{0,8}[1-9]))S))?",
        ) {
            prop_assert_eq!(s.clone(), RelativeDuration::parse_from_iso8601(&s).unwrap().format_to_iso8601());
        }

        #[test]
        fn proptest_parse_doesnt_panic(s in r"//PC*") {
            let _ = RelativeDuration::parse_from_iso8601(&s);
        }
    }
}
