//! How the home views write numbers, values and times: plain, server-side, the same on every
//! surface.

use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use serde_json::{Number, Value};

/// `value` with thousands separators and at most two decimals: `4,200`, `3.14`, `-12.5`.
pub(crate) fn number(value: f64) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    let negative = rounded < 0.0;
    let magnitude = rounded.abs();
    let whole = magnitude.trunc() as u64;
    let fraction = ((magnitude - magnitude.trunc()) * 100.0).round() as u64;
    let digits = whole.to_string();
    let mut grouped = String::new();
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(digit);
    }
    let fraction = match fraction {
        0 => String::new(),
        f if f % 10 == 0 => format!(".{}", f / 10),
        f => format!(".{f:02}"),
    };
    format!("{}{grouped}{fraction}", if negative { "-" } else { "" })
}

/// A shown value in `format` (`number`, `percent` or `currency` with its ISO code), then
/// `unit`: `4,200`, `42%`, `USD 4,200`, `120 ms`.
pub(crate) fn shown(
    value: &Value,
    format: Option<&str>,
    currency: Option<&str>,
    unit: Option<&str>,
) -> Result<String, String> {
    let core = match value {
        Value::Number(n) => {
            let n = n.as_f64().ok_or("the value is not a finite number")?;
            match (format, currency) {
                (Some("percent"), _) => format!("{}%", number(n)),
                (Some("currency"), Some(code)) => format!("{code} {}", number(n)),
                _ => number(n),
            }
        }
        Value::String(text) => text.clone(),
        Value::Bool(flag) => flag.to_string(),
        Value::Null => String::new(),
        _ => return Err("the value is a list or an object, not a number or text".to_string()),
    };
    Ok(match unit.map(str::trim).filter(|unit| !unit.is_empty()) {
        Some(unit) if !core.is_empty() => format!("{core} {unit}"),
        _ => core,
    })
}

/// A row's cell as a category's name.
pub(crate) fn cell_text(value: Option<&Value>, row: usize, key: &str) -> Result<String, String> {
    match value {
        Some(Value::String(text)) => Ok(text.clone()),
        Some(Value::Number(n)) => Ok(n.as_f64().map(number).unwrap_or_else(|| n.to_string())),
        Some(Value::Bool(flag)) => Ok(flag.to_string()),
        _ => Err(format!("row {row} has no {key} to name its place")),
    }
}

/// A row's cell as a number.
pub(crate) fn cell_number(value: Option<&Value>, row: usize, key: &str) -> Result<Number, String> {
    match value {
        Some(Value::Number(n)) => Ok(n.clone()),
        _ => Err(format!("row {row}'s {key} is not a number")),
    }
}

/// A row's cell as a time in epoch milliseconds: epoch milliseconds, an RFC 3339 time, or a
/// `YYYY-MM-DD` date (UTC).
pub(crate) fn cell_time(value: Option<&Value>, row: usize, key: &str) -> Result<i64, String> {
    let bad =
        || format!("row {row}'s {key} is not a time (epoch milliseconds, RFC 3339 or a date)");
    match value {
        Some(Value::Number(n)) => n
            .as_i64()
            .or_else(|| n.as_f64().map(|f| f as i64))
            .ok_or_else(bad),
        Some(Value::String(text)) => DateTime::parse_from_rfc3339(text)
            .map(|time| time.timestamp_millis())
            .or_else(|_| {
                NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S")
                    .map(|time| time.and_utc().timestamp_millis())
            })
            .or_else(|_| {
                NaiveDate::parse_from_str(text, "%Y-%m-%d").map(|date| {
                    date.and_hms_opt(0, 0, 0)
                        .map_or(0, |time| time.and_utc().timestamp_millis())
                })
            })
            .map_err(|_| bad()),
        _ => Err(bad()),
    }
}

/// Up to five evenly spaced ticks over `[low, high]` epoch milliseconds, written in `unit`
/// (`second`, `minute`, `hour`, `day`, `month`, `year`) or by the span when none is given.
pub(crate) fn time_ticks(low: i64, high: i64, unit: Option<&str>) -> Vec<(i64, String)> {
    const DAY: i64 = 86_400_000;
    let span = high.saturating_sub(low);
    let pattern = match unit {
        Some("second") => "%H:%M:%S",
        Some("minute" | "hour") => "%H:%M",
        Some("day") => "%b %-d",
        Some("month") => "%b %Y",
        Some("year") => "%Y",
        _ if span < DAY => "%H:%M",
        _ if span < 90 * DAY => "%b %-d",
        _ if span < 3 * 365 * DAY => "%b %Y",
        _ => "%Y",
    };
    let count: i64 = if span == 0 { 1 } else { 5 };
    let mut ticks: Vec<(i64, String)> = Vec::new();
    for step in 0..count {
        let at = if count == 1 {
            low
        } else {
            low + span * step / (count - 1)
        };
        let Some(time) = DateTime::<Utc>::from_timestamp_millis(at) else {
            continue;
        };
        let text = time.format(pattern).to_string();
        // Neighbouring ticks that read the same say nothing new.
        if ticks.last().is_none_or(|(_, last)| *last != text) {
            ticks.push((at, text));
        }
    }
    ticks
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn numbers_read_plainly() {
        assert_eq!(number(4200.0), "4,200");
        assert_eq!(number(1_234_567.891), "1,234,567.89");
        assert_eq!(number(-12.5), "-12.5");
        assert_eq!(number(0.0), "0");
        assert_eq!(number(1.23456), "1.23");
    }

    #[test]
    fn values_take_their_format_and_unit() {
        assert_eq!(
            shown(&json!(42), Some("percent"), None, None).unwrap(),
            "42%"
        );
        assert_eq!(
            shown(&json!(4200), Some("currency"), Some("USD"), None).unwrap(),
            "USD 4,200"
        );
        assert_eq!(
            shown(&json!(120), None, None, Some("ms")).unwrap(),
            "120 ms"
        );
        assert_eq!(
            shown(&json!("n/a"), None, None, Some("ms")).unwrap(),
            "n/a ms"
        );
        assert!(shown(&json!([1]), None, None, None).is_err());
    }

    #[test]
    fn times_parse_from_every_accepted_form() {
        let expected = 1_763_251_200_000; // 2025-11-16T00:00:00Z
        assert_eq!(cell_time(Some(&json!(expected)), 0, "t").unwrap(), expected);
        assert_eq!(
            cell_time(Some(&json!("2025-11-16T00:00:00Z")), 0, "t").unwrap(),
            expected
        );
        assert_eq!(
            cell_time(Some(&json!("2025-11-16")), 0, "t").unwrap(),
            expected
        );
        assert!(cell_time(Some(&json!("yesterday")), 0, "t").is_err());
    }

    #[test]
    fn ticks_follow_the_span() {
        let day = 86_400_000;
        let ticks = time_ticks(1_763_251_200_000, 1_763_251_200_000 + 6 * day, None);
        assert_eq!(ticks.first().unwrap().1, "Nov 16");
        assert!(ticks.len() <= 5);
        assert_eq!(
            time_ticks(0, 0, Some("year")),
            vec![(0, "1970".to_string())]
        );
    }
}
