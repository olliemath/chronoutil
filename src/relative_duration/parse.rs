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
}
