use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};

// Parser for -d "YYYY-MM-DDThh:mm:SS[.frac][tz]"
fn parse_d_format(s: &str) -> anyhow::Result<DateTime<Utc>> {
    // RFC3339 requires 'T', but `touch -d` allows ' '
    let s_normalized = if s.contains(' ') && !s.contains('T') {
        s.replace(' ', "T")
    } else {
        s.to_string()
    };

    // Try parsing as DateTime<Utc> directly if 'Z' is present
    // or as DateTime<Local> if no timezone info (then convert to Utc)
    // `parse_from_rfc3339` handles these cases and optional fractional seconds.
    match DateTime::parse_from_rfc3339(&s_normalized) {
        Ok(dt) => Ok(dt.with_timezone(&Utc)),
        Err(e) => anyhow::bail!("Invalid -d format [{}]: {}", s, e),
    }
}

// Parser for -t "[[CC]YY]MMDDhhmm[.SS]"
fn parse_t_format(s: &str) -> anyhow::Result<DateTime<Utc>> {
    let now = Local::now();
    let current_year = now.year();
    let (rest, ss) = if let Some((main_part, sec_part)) = s.split_once('.') {
        if sec_part.len() > 2 || sec_part.parse::<u32>().is_err() {
            anyhow::bail!("Invalid seconds in -t format: {}", sec_part);
        }
        let ss_val = sec_part.parse::<u32>()?;
        if ss_val > 60 {
            // up to 60 for leap seconds
            anyhow::bail!("Seconds out of range (0-60): {}", ss_val);
        }
        (main_part, ss_val)
    } else {
        (s, 0) // Default seconds to 00
    };

    let len = rest.len();
    if !(matches!(len, 8 | 10 | 12)) || !rest.chars().all(|c| c.is_ascii_digit()) {
        anyhow::bail!(
            "Invalid -t format string: {}. Expected [[CC]YY]MMDDhhmm, got: {}",
            s,
            rest
        );
    }

    // Extract MMDDhhmm (guaranteed to be 8 digits from right)
    let mm_str = &rest[len - 4..len - 2];
    let hh_str = &rest[len - 6..len - 4];
    let dd_str = &rest[len - 8..len - 6];
    let mon_str = &rest[len - 10..len - 8];

    let mm_val = mm_str.parse::<u32>()?;
    let hh_val = hh_str.parse::<u32>()?;
    let dd_val = dd_str.parse::<u32>()?;
    let mon_val = mon_str.parse::<u32>()?;

    // Year part is the prefix
    let year_part_str = &rest[0..len - 8];
    let year_val = match year_part_str.len() {
        0 => current_year, // No year specified, use current
        2 => {
            // YY
            let yy = year_part_str.parse::<i32>()?;
            if yy >= 69 && yy <= 99 {
                1900 + yy
            } else {
                2000 + yy
            }
        }
        4 => {
            // CCYY
            year_part_str.parse::<i32>()?
        }
        _ => anyhow::bail!("Invalid year in -t format: {}", year_part_str),
    };

    let naive_date = NaiveDate::from_ymd_opt(year_val, mon_val, dd_val)
        .ok_or_else(|| anyhow::anyhow!("Invalid date: {}-{}-{}", year_val, mon_val, dd_val))?;
    let naive_time = NaiveTime::from_hms_opt(hh_val, mm_val, ss)
        .ok_or_else(|| anyhow::anyhow!("Invalid time: {}:{}:{}", hh_val, mm_val, ss))?;

    let naive_datetime = NaiveDateTime::new(naive_date, naive_time);

    // Assume local time, then convert to UTC
    Ok(Local
        .from_local_datetime(&naive_datetime)
        .single() // Handle DST transitions: choose one unique mapping.
        .ok_or_else(|| anyhow::anyhow!("Ambiguous or non-existent local time: {}", naive_datetime))?
        .with_timezone(&Utc))
}
