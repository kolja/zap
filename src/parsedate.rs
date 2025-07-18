use crate::ZapError;
use chrono::{DateTime, Datelike, Local, NaiveDateTime, TimeZone, Timelike, Utc};

// Parser for -d "YYYY-MM-DDThh:mm:SS[.frac][tz]"
pub fn parse_d_format(s: &str) -> anyhow::Result<DateTime<Utc>> {
    // first try RFC3339 for inputs with a timezone offset.
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }

    if let Ok(naive_dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f") {
        let local_dt = Local
            .from_local_datetime(&naive_dt)
            .single()
            .ok_or_else(||
                ZapError::ParseRfc3339 {
                    input: s.to_string(),
                    reason: "Failed to convert local time".to_string(),
                }
            )?;
        return Ok(local_dt.with_timezone(&Utc));
    }
    Err(ZapError::ParseRfc3339 {
        input: s.to_string(),
        reason: "Invalid date-time format, expected RFC3339 or YYYY-MM-DDThh:mm:SS[.frac]".to_string(),
    })?
}

// Parser for -t "[[CC]YY]MMDDhhmm[.SS]"
pub fn parse_t_format(s: &str) -> anyhow::Result<DateTime<Utc>> {
    let parts: Vec<&str> = s.split('.').collect();
    let (date_time_str, sec_str) = match parts.as_slice() {
        [dt] => (*dt, "0"), // No seconds provided, default to 0.
        [dt, ss] if ss.len() == 2 => (*dt, *ss),
        _ => {
            return Err(ZapError::ParseTOption {
                input: s.to_string(),
                reason: format!("format must be [[CC]YY]MMDDhhmm[.SS]"),
            }
            .into());
        }
    };

    let second = sec_str
        .parse::<u32>()
        .map_err(|_| ZapError::TOptionInvalidSecondString { second: sec_str.to_string() })?;

    let naive_dt_base =
        match date_time_str.len() {
            // MMDDhhmm: Prepend the current year and parse.
            8 => {
                let s_with_year = format!("{}{}", Local::now().year(), date_time_str);
                NaiveDateTime::parse_from_str(&s_with_year, "%Y%m%d%H%M")
            }
            // YYMMDDhhmm: The %y format specifier correctly handles the 1969-2068 rule.
            10 => NaiveDateTime::parse_from_str(date_time_str, "%y%m%d%H%M"),
            // CCYYMMDDhhmm:
            12 => NaiveDateTime::parse_from_str(date_time_str, "%Y%m%d%H%M"),
            _ => return Err(ZapError::TOptionWrongLength { length: date_time_str.len() }.into()),
        }
        .map_err(|e| ZapError::ParseTOption {
            input: s.to_string(),
            reason: e.to_string(),
        })?;

    let naive_dt = naive_dt_base
        .with_second(second)
        .ok_or_else(|| ZapError::TOptionInvalidSecond { second })?;

    let local_dt = Local
        .from_local_datetime(&naive_dt)
        .single()
        .ok_or_else(|| ZapError::TOptionConvertToLocal )?;

    Ok(local_dt.with_timezone(&Utc))
}

// Parser for -A "[-][[hh]mm]SS"
pub fn parse_adjust(s: &str) -> Result<i32, anyhow::Error> {
    let sign = if s.chars().next().unwrap_or('+') == '-' {
        -1
    } else {
        1
    };

    // 2, 4 or 6 digit number as string ([-][[hh]mm]SS)
    let num = s.strip_prefix('-').unwrap_or(s);

    debug_assert!(num.is_ascii() && num.len() % 2 == 0);

    let sum: i32 = (0..num.len())
        .step_by(2)
        .map(|i| {
            let chunk = &num[i..i + 2];
            chunk.parse::<i32>()
                 .map_err(|e| ZapError::ParseAdjustment {reason: e.to_string()})
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .rev() // Reverse the iterator of parsed numbers.
        .zip([1, 60, 3600])
        .map(|(val, mult)| val * mult)
        .sum();

    Ok(sign * sum)
}
