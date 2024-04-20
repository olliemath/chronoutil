use super::RelativeDuration;
use chrono::Duration;

fn dhmsn_to_duration(days: i32, hours: i32, minutes: i32, seconds: i32, nanos: i32) -> Duration {
    Duration::seconds((((days * 24 + hours) * 60 + minutes) * 60 + seconds) as i64)
        + Duration::nanoseconds(nanos.into())
}

fn get_terminated(input: &str, terminator: char) -> Result<(&str, i32), String> {
    if let Some((int_string, remainder)) = input.split_once(terminator) {
        let int = int_string
            .parse::<i32>()
            .map_err(|_| format!("{} is not a valid i32", int_string))?;
        Ok((remainder, int))
    } else {
        Ok((input, 0))
    }
}

fn get_terminated_decimal(input: &str, terminator: char) -> Result<(&str, i32, i32), String> {
    if let Some((decimal_string, remainder)) = input.split_once(terminator) {
        let (int_string, fraction_string) = decimal_string.split_once('.').unwrap_or_else(|| {
            decimal_string
                // if no '.' was found, look for ',', as both are valid decimal separators in iso 8601
                .split_once(',')
                // if neither is found take the whole string as the integer part, with no fraction
                .unwrap_or((decimal_string, ""))
        });

        let int = int_string
            .parse::<i32>()
            .map_err(|_| format!("{} is not a valid i32", int_string))?;

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
                .parse::<i32>()
                .map_err(|_| format!("{} is not a valid i32", fraction_string))?
        };
        Ok((remainder, int, fraction))
    } else {
        Ok((input, 0, 0))
    }
}

fn parse_datespec(datespec: &str) -> Result<(i32, i32, i32), String> {
    let (remainder, years) = get_terminated(datespec, 'Y')?;
    let (remainder, months) = get_terminated(remainder, 'M')?;
    let (remainder, weeks) = get_terminated(remainder, 'W')?;
    let (remainder, days) = get_terminated(remainder, 'D')?;

    if !remainder.is_empty() {
        Err(format!(
            "trailing characters: {} in datespec: {}",
            remainder, datespec
        ))
    } else {
        Ok((years, months, (weeks * 7) + days))
    }
}

fn parse_timespec(timespec: &str) -> Result<(i32, i32, i32, i32), String> {
    let (remainder, hours) = get_terminated(timespec, 'H')?;
    let (remainder, mins) = get_terminated(remainder, 'M')?;
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

        Ok(RelativeDuration::months(years * 12 + months)
            .with_duration(dhmsn_to_duration(days, hours, mins, secs, nanos)))
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

        // TODO: address this unwrap
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
        if !time_spec.is_empty() {
            out.push('T');
            out.push_str(&time_spec);
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
                    .with_duration(dhmsn_to_duration(2, 2, 2, 2, 0)),
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
                    .with_duration(dhmsn_to_duration(2, 2, 2, 2, 0)),
                "P2Y2M2DT2H2M2S",
            ),
            (
                RelativeDuration::months(1).with_duration(Duration::zero()),
                "P1M",
            ),
            (RelativeDuration::minutes(10), "PT10M"),
            (RelativeDuration::months(-1), "P-1M"),
        ]
        .iter()
        .for_each(|(input, expected)| assert_eq!(input.to_iso_8601(), *expected))
    }
}