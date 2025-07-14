use chrono::{DateTime, Datelike, Local, NaiveDateTime, Timelike, TimeZone, Utc};

// Parser for -d "YYYY-MM-DDThh:mm:SS[.frac][tz]"
pub fn parse_d_format(s: &str) -> anyhow::Result<DateTime<Utc>> {
    // first try RFC3339 for inputs with a timezone offset.
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }
    // then fallback to a naive datetime format, assuming local time.
    // The `%.f` specifier handles optional fractional seconds.
    if let Ok(naive_dt) = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f") {
        let local_dt = Local.from_local_datetime(&naive_dt).single()
            .ok_or_else(|| anyhow::anyhow!("Failed to convert local time from: {}", s))?;
        return Ok(local_dt.with_timezone(&Utc));
    }
    anyhow::bail!("Invalid -d format: {}", s)
}

// Parser for -t "[[CC]YY]MMDDhhmm[.SS]"
pub fn parse_t_format(s: &str) -> anyhow::Result<DateTime<Utc>> {

    let parts: Vec<&str> = s.split('.').collect();
    let (date_time_str, sec_str) = match parts.as_slice() {
        [dt] => (*dt, "0"), // No seconds provided, default to 0.
        [dt, ss] if ss.len() == 2 => (*dt, *ss),
        _ => anyhow::bail!("Invalid -t format: must be [[CC]YY]MMDDhhmm[.SS]"),
    };

    let second = sec_str.parse::<u32>()
        .map_err(|_| anyhow::anyhow!("Invalid seconds in -t format: {}", s))?;

    let naive_dt_base = match date_time_str.len() {
        // MMDDhhmm: Prepend the current year and parse.
        8 => {
            let s_with_year = format!("{}{}", Local::now().year(), date_time_str);
            NaiveDateTime::parse_from_str(&s_with_year, "%Y%m%d%H%M")
        }
        // YYMMDDhhmm: The %y format specifier correctly handles the 1969-2068 rule.
        10 => NaiveDateTime::parse_from_str(date_time_str, "%y%m%d%H%M"),
        // CCYYMMDDhhmm:
        12 => NaiveDateTime::parse_from_str(date_time_str, "%Y%m%d%H%M"),
        _ => {
            return Err(anyhow::anyhow!("Invalid date/time format length: {}", date_time_str.len()));
        }
    }
    .map_err(|_| anyhow::anyhow!("Invalid date/time format for {}", date_time_str))?;

    let naive_dt = naive_dt_base
        .with_second(second)
        .ok_or_else(|| anyhow::anyhow!("Invalid second value: {}", second))?;

    let local_dt = Local
        .from_local_datetime(&naive_dt)
        .single()
        .ok_or_else(|| anyhow::anyhow!("Failed to convert local time for {}", s))?;

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

    let sum: i32 = num.as_bytes()
                      .chunks(2)
                      .map(|chunk| unsafe {
                          // This parse can be fallible. Assuming valid digits for now.
                          std::str::from_utf8_unchecked(chunk).parse::<i32>().unwrap()
                      })
                      .rev() // Reverse the iterator of parsed numbers.
                      .zip([1, 60, 3600])
                      .map(|(val, mult)| val * mult)
                      .sum();

    Ok(sign * sum)
}
