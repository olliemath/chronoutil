use super::RelativeDuration;
use chrono::Duration;

fn dhms_to_duration(days: i32, hours: i32, minutes: i32, seconds: i32) -> Duration {
    Duration::seconds((((days * 24 + hours) * 60 + minutes) * 60 + seconds) as i64)
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

fn parse_timespec(timespec: &str) -> Result<(i32, i32, i32), String> {
    let (remainder, hours) = get_terminated(timespec, 'H')?;
    let (remainder, mins) = get_terminated(remainder, 'M')?;
    let (remainder, secs) = get_terminated(remainder, 'S')?;

    if !remainder.is_empty() {
        Err(format!(
            "trailing characters: {} in timespec: {}",
            remainder, timespec
        ))
    } else {
        Ok((hours, mins, secs))
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
    pub fn from_iso_8601(input: &str) -> Result<RelativeDuration, String> {
        let input = input
            .strip_prefix('P')
            .ok_or_else(|| "duration was not prefixed with P".to_string())?;

        let (datespec, timespec) = input.split_once('T').unwrap_or((input, ""));

        let (years, months, days) = parse_datespec(datespec)?;
        let (hours, mins, secs) = parse_timespec(timespec)?;

        Ok(RelativeDuration::months(years * 12 + months)
            .with_duration(dhms_to_duration(days, hours, mins, secs)))
    }

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
                RelativeDuration::months(2 * 12 + 2).with_duration(dhms_to_duration(2, 2, 2, 2)),
            ),
            (
                "P1M",
                RelativeDuration::months(1).with_duration(Duration::zero()),
            ),
            ("PT10M", RelativeDuration::minutes(10)),
            ("P-1M", RelativeDuration::months(-1)),
            ("P1W1D", RelativeDuration::days(8)),
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
                RelativeDuration::months(2 * 12 + 2).with_duration(dhms_to_duration(2, 2, 2, 2)),
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
