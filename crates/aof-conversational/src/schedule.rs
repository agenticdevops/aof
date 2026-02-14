use chrono::{DateTime, Utc};
use cron::Schedule;
use regex::Regex;
use std::str::FromStr;
use thiserror::Error;

/// Errors that can occur during schedule parsing
#[derive(Debug, Error)]
pub enum ScheduleError {
    #[error("Could not parse schedule expression: {0}")]
    UnparsableInput(String),

    #[error("Invalid cron expression: {0}")]
    InvalidCron(String),

    #[error("Invalid timezone: {0}")]
    InvalidTimezone(String),

    #[error("LLM error: {0}")]
    LlmError(String),
}

/// A parsed schedule with validation information
#[derive(Debug, Clone)]
pub struct ParsedSchedule {
    /// Standard 6-field cron expression (with seconds)
    pub cron_expression: String,
    /// IANA timezone string (e.g., "America/New_York")
    pub timezone: String,
    /// Human-readable description
    pub description: String,
    /// Next 3 scheduled runs
    pub next_runs: Vec<DateTime<Utc>>,
}

/// Parse a natural language schedule description into a cron expression
///
/// Supports patterns like:
/// - "every 30 minutes" -> `0 */30 * * * *`
/// - "every 5 hours" -> `0 0 */5 * * *`
/// - "daily at 6am" -> `0 0 6 * * *`
/// - "every weekday at 9am" -> `0 0 9 * * 1-5`
/// - "business hours" -> `0 0 9-17 * * 1-5`
/// - "3 times per day" -> `0 0 6,12,18 * * *`
///
/// Timezone can be specified at the end: "daily at 6am EST"
pub fn parse_natural_schedule(input: &str) -> Result<ParsedSchedule, ScheduleError> {
    // Extract timezone first
    let (cleaned_input, timezone) = extract_timezone(input);
    let lower = cleaned_input.to_lowercase();

    // Try to match patterns
    let cron_expr = match_schedule_pattern(&lower)?;

    // Validate and get next runs
    let next_runs = validate_cron(&cron_expr, &timezone)?;

    Ok(ParsedSchedule {
        cron_expression: cron_expr.clone(),
        timezone: timezone.clone(),
        description: input.to_string(),
        next_runs,
    })
}

/// Match natural language patterns to cron expressions
fn match_schedule_pattern(input: &str) -> Result<String, ScheduleError> {
    // Pattern 1: every N minutes
    let minutes_re = Regex::new(r"every\s+(\d+)\s+minutes?").unwrap();
    if let Some(caps) = minutes_re.captures(input) {
        let n: u32 = caps[1].parse().map_err(|_| {
            ScheduleError::UnparsableInput("Invalid minute value".to_string())
        })?;
        return Ok(format!("0 */{} * * * *", n));
    }

    // Pattern 2: every N hours
    let hours_re = Regex::new(r"every\s+(\d+)\s+hours?").unwrap();
    if let Some(caps) = hours_re.captures(input) {
        let n: u32 = caps[1].parse().map_err(|_| {
            ScheduleError::UnparsableInput("Invalid hour value".to_string())
        })?;
        return Ok(format!("0 0 */{} * * *", n));
    }

    // Pattern 3: business hours
    if input.contains("business hours") {
        return Ok("0 0 9-17 * * 1-5".to_string());
    }

    // Pattern 4: N times per day (evenly distributed)
    let times_per_day_re = Regex::new(r"(\d+)x?\s+(?:times?\s+)?per\s+day").unwrap();
    if let Some(caps) = times_per_day_re.captures(input) {
        let n: u32 = caps[1].parse().map_err(|_| {
            ScheduleError::UnparsableInput("Invalid times per day value".to_string())
        })?;
        if n == 0 || n > 24 {
            return Err(ScheduleError::UnparsableInput(
                "Times per day must be between 1 and 24".to_string(),
            ));
        }
        let hours = distribute_hours(n);
        return Ok(format!("0 0 {} * * *", hours));
    }

    // Pattern 5: daily at specific time
    let daily_re = Regex::new(
        r"daily\s+at\s+(\d{1,2})(?::(\d{2}))?\s*(am|pm|noon|midnight)?",
    )
    .unwrap();
    if let Some(caps) = daily_re.captures(input) {
        let hour_str = &caps[1];
        let min_str = caps.get(2).map(|m| m.as_str()).unwrap_or("0");
        let period = caps.get(3).map(|m| m.as_str());

        let mut hour: u32 = hour_str.parse().map_err(|_| {
            ScheduleError::UnparsableInput("Invalid hour".to_string())
        })?;
        let min: u32 = min_str.parse().map_err(|_| {
            ScheduleError::UnparsableInput("Invalid minute".to_string())
        })?;

        // Handle AM/PM
        if let Some(p) = period {
            match p {
                "pm" if hour != 12 => hour += 12,
                "am" if hour == 12 => hour = 0,
                "noon" => hour = 12,
                "midnight" => hour = 0,
                _ => {}
            }
        }

        if hour > 23 || min > 59 {
            return Err(ScheduleError::UnparsableInput(
                "Invalid time values".to_string(),
            ));
        }

        return Ok(format!("0 {} {} * * *", min, hour));
    }

    // Pattern 6: noon/midnight special cases
    if input.contains("noon") && input.contains("daily") {
        return Ok("0 0 12 * * *".to_string());
    }
    if input.contains("midnight") && input.contains("daily") {
        return Ok("0 0 0 * * *".to_string());
    }

    // Pattern 7: every weekday (optionally with time)
    if input.contains("weekday") {
        // Check if there's a time component
        let time_re = Regex::new(r"at\s+(\d{1,2})(?::(\d{2}))?\s*(am|pm)?").unwrap();
        if let Some(caps) = time_re.captures(input) {
            let mut hour: u32 = caps[1].parse().unwrap_or(0);
            let min: u32 = caps.get(2).map(|m| m.as_str()).unwrap_or("0").parse().unwrap_or(0);
            if let Some(period) = caps.get(3).map(|m| m.as_str()) {
                if period == "pm" && hour != 12 {
                    hour += 12;
                } else if period == "am" && hour == 12 {
                    hour = 0;
                }
            }
            return Ok(format!("0 {} {} * * 1-5", min, hour));
        }
        return Ok("0 0 0 * * 1-5".to_string());
    }

    // Pattern 8: specific days (Monday, Tuesday, etc.)
    let days_map = [
        ("monday", "1"),
        ("tuesday", "2"),
        ("wednesday", "3"),
        ("thursday", "4"),
        ("friday", "5"),
        ("saturday", "6"),
        ("sunday", "0"),
    ];

    let mut matched_days = Vec::new();
    for (day_name, day_num) in &days_map {
        if input.contains(day_name) {
            matched_days.push(*day_num);
        }
    }

    if !matched_days.is_empty() {
        let days_str = matched_days.join(",");
        // Check for time component
        let time_re = Regex::new(r"at\s+(\d{1,2})(?::(\d{2}))?\s*(am|pm)?").unwrap();
        if let Some(caps) = time_re.captures(input) {
            let mut hour: u32 = caps[1].parse().unwrap_or(0);
            let min: u32 = caps.get(2).map(|m| m.as_str()).unwrap_or("0").parse().unwrap_or(0);
            if let Some(period) = caps.get(3).map(|m| m.as_str()) {
                if period == "pm" && hour != 12 {
                    hour += 12;
                } else if period == "am" && hour == 12 {
                    hour = 0;
                }
            }
            return Ok(format!("0 {} {} * * {}", min, hour, days_str));
        }
        return Ok(format!("0 0 0 * * {}", days_str));
    }

    // No pattern matched
    Err(ScheduleError::UnparsableInput(
        "Try something like 'every 30 minutes' or 'daily at 6am EST'".to_string(),
    ))
}

/// Distribute N times across 24 hours evenly
fn distribute_hours(n: u32) -> String {
    if n == 1 {
        return "12".to_string();
    }
    if n == 2 {
        return "6,18".to_string();
    }
    if n == 3 {
        return "6,12,18".to_string();
    }
    if n == 4 {
        return "6,12,18,24".to_string();
    }

    let interval = 24 / n;
    let hours: Vec<String> = (0..n).map(|i| (i * interval).to_string()).collect();
    hours.join(",")
}

/// Extract timezone from input string
///
/// Returns (cleaned_input, timezone)
/// Recognizes: EST, CST, MST, PST, UTC, and IANA names
pub fn extract_timezone(input: &str) -> (String, String) {
    let tz_map = [
        ("EST", "America/New_York"),
        ("EDT", "America/New_York"),
        ("CST", "America/Chicago"),
        ("CDT", "America/Chicago"),
        ("MST", "America/Denver"),
        ("MDT", "America/Denver"),
        ("PST", "America/Los_Angeles"),
        ("PDT", "America/Los_Angeles"),
        ("UTC", "UTC"),
    ];

    // Check for abbreviated timezones
    for (abbr, tz) in &tz_map {
        if input.to_uppercase().ends_with(abbr) {
            let cleaned = input[..input.len() - abbr.len()].trim().to_string();
            return (cleaned, tz.to_string());
        }
        // Also check with whitespace before timezone
        let pattern = format!(" {}", abbr);
        if input.to_uppercase().contains(&pattern) {
            let cleaned = input.to_uppercase().replace(&pattern, "");
            return (cleaned, tz.to_string());
        }
    }

    // Check for IANA timezone format (e.g., "Europe/London")
    let iana_re = Regex::new(r"\s+([A-Z][a-zA-Z_]+/[A-Za-z_]+)$").unwrap();
    if let Some(caps) = iana_re.captures(input) {
        let tz = caps[1].to_string();
        let cleaned = input[..caps.get(1).unwrap().start()].trim().to_string();
        return (cleaned, tz);
    }

    // Default to UTC
    (input.to_string(), "UTC".to_string())
}

/// Validate a cron expression and return next 3 scheduled runs
pub fn validate_cron(cron_expr: &str, timezone: &str) -> Result<Vec<DateTime<Utc>>, ScheduleError> {
    // Parse cron expression
    let schedule = Schedule::from_str(cron_expr)
        .map_err(|e| ScheduleError::InvalidCron(format!("{}: {}", cron_expr, e)))?;

    // Parse timezone
    let tz: chrono_tz::Tz = timezone
        .parse()
        .map_err(|_| ScheduleError::InvalidTimezone(timezone.to_string()))?;

    // Get next 3 runs

    let next_runs: Vec<DateTime<Utc>> = schedule
        .upcoming(tz)
        .take(3)
        .map(|dt| dt.with_timezone(&Utc))
        .collect();

    if next_runs.is_empty() {
        return Err(ScheduleError::InvalidCron(
            "Cron expression produces no future runs".to_string(),
        ));
    }

    Ok(next_runs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_every_30_minutes() {
        let result = parse_natural_schedule("every 30 minutes").unwrap();
        assert_eq!(result.cron_expression, "0 */30 * * * *");
        assert_eq!(result.timezone, "UTC");
        assert_eq!(result.next_runs.len(), 3);
    }

    #[test]
    fn test_every_5_hours() {
        let result = parse_natural_schedule("every 5 hours").unwrap();
        assert_eq!(result.cron_expression, "0 0 */5 * * *");
    }

    #[test]
    fn test_daily_at_6am() {
        let result = parse_natural_schedule("daily at 6am").unwrap();
        assert_eq!(result.cron_expression, "0 0 6 * * *");
    }

    #[test]
    fn test_daily_at_6pm() {
        let result = parse_natural_schedule("daily at 6pm").unwrap();
        assert_eq!(result.cron_expression, "0 0 18 * * *");
    }

    #[test]
    fn test_daily_at_noon() {
        let result = parse_natural_schedule("daily at noon").unwrap();
        assert_eq!(result.cron_expression, "0 0 12 * * *");
    }

    #[test]
    fn test_daily_at_midnight() {
        let result = parse_natural_schedule("daily at midnight").unwrap();
        assert_eq!(result.cron_expression, "0 0 0 * * *");
    }

    #[test]
    fn test_every_weekday_at_9am() {
        let result = parse_natural_schedule("every weekday at 9am").unwrap();
        assert_eq!(result.cron_expression, "0 0 9 * * 1-5");
    }

    #[test]
    fn test_every_monday_and_friday() {
        let result = parse_natural_schedule("every monday and friday").unwrap();
        assert!(result.cron_expression.contains("1") && result.cron_expression.contains("5"));
    }

    #[test]
    fn test_business_hours() {
        let result = parse_natural_schedule("business hours").unwrap();
        assert_eq!(result.cron_expression, "0 0 9-17 * * 1-5");
    }

    #[test]
    fn test_3_times_per_day() {
        let result = parse_natural_schedule("3 times per day").unwrap();
        assert_eq!(result.cron_expression, "0 0 6,12,18 * * *");
    }

    #[test]
    fn test_est_timezone() {
        let result = parse_natural_schedule("daily at 6am EST").unwrap();
        assert_eq!(result.timezone, "America/New_York");
        assert_eq!(result.cron_expression, "0 0 6 * * *");
    }

    #[test]
    fn test_pst_timezone() {
        let result = parse_natural_schedule("daily at 3pm PST").unwrap();
        assert!(result.timezone.contains("Los_Angeles"));
    }

    #[test]
    fn test_utc_default() {
        let result = parse_natural_schedule("every 30 minutes").unwrap();
        assert_eq!(result.timezone, "UTC");
    }

    #[test]
    fn test_iana_timezone() {
        let (cleaned, tz) = extract_timezone("daily at 9am Europe/London");
        assert_eq!(tz, "Europe/London");
        assert_eq!(cleaned, "daily at 9am");
    }

    #[test]
    fn test_validate_cron_returns_3_runs() {
        let runs = validate_cron("0 */30 * * * *", "UTC").unwrap();
        assert_eq!(runs.len(), 3);
    }

    #[test]
    fn test_invalid_cron_rejected() {
        let result = validate_cron("invalid cron", "UTC");
        assert!(result.is_err());
    }

    #[test]
    fn test_24_hour_format() {
        let result = parse_natural_schedule("daily at 14:30").unwrap();
        assert_eq!(result.cron_expression, "0 30 14 * * *");
    }

    #[test]
    fn test_12am_is_midnight() {
        let result = parse_natural_schedule("daily at 12am").unwrap();
        assert_eq!(result.cron_expression, "0 0 0 * * *");
    }

    #[test]
    fn test_12pm_is_noon() {
        let result = parse_natural_schedule("daily at 12pm").unwrap();
        assert_eq!(result.cron_expression, "0 0 12 * * *");
    }
}
