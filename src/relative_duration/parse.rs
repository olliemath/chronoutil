use super::RelativeDuration;
use chrono::Duration;

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
        Ok((remainder, int, fraction))
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

fn format_spec(nums: [i64; 3], chars: [char; 3]) -> String {
    nums.iter()
        .zip(chars)
        .filter(|x| *x.0 != 0)
        .map(|x| format!("{}{}", x.0, x.1))
        .reduce(|acc, x| acc + &x)
        .unwrap_or_else(|| "".to_string())
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
    ///     RelativeDuration::from_iso_8601("P1Y").unwrap(),
    ///     RelativeDuration::years(1),
    /// );
    /// ```
    pub fn from_iso_8601(input: &str) -> Result<RelativeDuration, String> {
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
    ///     RelativeDuration::years(1).to_iso_8601(),
    ///     "P1Y",
    /// );
    /// ```
    pub fn to_iso_8601(&self) -> String {
        let years = self.months as i64 / 12;
        let months = self.months as i64 % 12;

        let duration_seconds = self.duration.num_seconds();

        let days = duration_seconds / (24 * 60 * 60);
        let mut remaining_seconds = duration_seconds - (days * 24 * 60 * 60);

        let hours = remaining_seconds / (60 * 60);
        remaining_seconds -= hours * 60 * 60;

        let minutes = remaining_seconds / 60;
        remaining_seconds -= minutes * 60;

        let seconds = remaining_seconds;

        let date_spec = format_spec([years, months, days], ['Y', 'M', 'D']);
        let time_spec = format_spec([hours, minutes, seconds], ['H', 'M', 'S']);

        let mut out = String::with_capacity(date_spec.len() + time_spec.len() + 2);
        out.push('P');
        out.push_str(&date_spec);
        if !time_spec.is_empty() || self.duration.subsec_nanos() != 0 {
            out.push('T');
            if time_spec.is_empty() {
                out.push('0');
            } else {
                out.push_str(&time_spec);
            }
        }
        if self.duration.subsec_nanos() != 0 {
            out = out.trim_end_matches('S').to_string();
            let nanos_str_raw = format!("{:0>9}", self.duration.subsec_nanos());
            let nanos_str_trimmed = nanos_str_raw.trim_end_matches('0');
            out.push('.');
            out.push_str(nanos_str_trimmed);
            out.push('S');
        }

        out
    }
}

#[cfg(test)]
mod tests {
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
            ("PT0.0000000010S", RelativeDuration::nanoseconds(1)),
            ("PT0.1S", RelativeDuration::nanoseconds(100_000_000)),
        ]
        .iter()
        .for_each(|(input, expected)| {
            assert_eq!(RelativeDuration::from_iso_8601(input).unwrap(), *expected)
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
            (RelativeDuration::nanoseconds(1), "PT0.000000001S"),
            (RelativeDuration::nanoseconds(100_000_000), "PT0.1S"),
        ]
        .iter()
        .for_each(|(input, expected)| assert_eq!(input.to_iso_8601(), *expected))
    }
}
