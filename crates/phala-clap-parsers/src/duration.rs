#[derive(Debug)]
pub struct InvalidDuration;

impl std::fmt::Display for InvalidDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid duration")
    }
}

impl std::error::Error for InvalidDuration {}

use core::time::Duration;

pub fn parse_duration(s: &str) -> Result<Duration, InvalidDuration> {
    let mut num_str = s;
    let mut unit = "s";

    if let Some(idx) = s.find(|c: char| !c.is_numeric()) {
        num_str = &s[..idx];
        unit = &s[idx..];
    }

    let num = num_str.parse::<u64>().or(Err(InvalidDuration))?;

    let num = match unit {
        "s" | "" => num * 1000,
        "m" => num * 60 * 1000,
        "h" => num * 60 * 60 * 1000,
        "d" => num * 60 * 60 * 24 * 1000,
        "ms" => num,
        _ => return Err(InvalidDuration),
    };

    Ok(Duration::from_millis(num))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("10ms").unwrap(), Duration::from_millis(10));
        assert_eq!(parse_duration("10m").unwrap(), Duration::from_secs(600));
        assert_eq!(parse_duration("10s").unwrap(), Duration::from_secs(10));
        assert_eq!(parse_duration("10m").unwrap(), Duration::from_secs(600));
        assert_eq!(parse_duration("10h").unwrap(), Duration::from_secs(36000));
        assert_eq!(parse_duration("10d").unwrap(), Duration::from_secs(864000));
        assert_eq!(parse_duration("1").unwrap(), Duration::from_secs(1));
        assert_eq!(parse_duration("100").unwrap(), Duration::from_secs(100));
    }

    #[test]
    fn test_parse_duration_invalid() {
        assert!(parse_duration("10x").is_err());
        assert!(parse_duration("ms").is_err());
    }

    #[test]
    fn test_parse_duration_empty() {
        assert!(parse_duration("").is_err());
    }
}
