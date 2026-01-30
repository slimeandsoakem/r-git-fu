use crate::primitives::FuError;
use chrono::{DateTime, TimeZone, Utc};
use std::fmt::Display;
use std::io::{self, Write};

pub fn safe_println(s: &str) {
    if let Err(e) = writeln!(io::stdout(), "{}", s) {
        if e.kind() != io::ErrorKind::BrokenPipe {
            panic!("stdout error: {}", e);
        }
        std::process::exit(0);
    }
}

pub fn timestamp_to_datetime(ts: i64) -> Result<DateTime<Utc>, FuError> {
    let timestamp = Utc
        .timestamp_opt(ts, 0)
        .single()
        .ok_or(FuError::Custom("Time out of range".to_string()))?;
    Ok(timestamp)
}
pub fn format_commit_time(ts: i64) -> Result<(String, String), FuError> {
    let datetime = timestamp_to_datetime(ts)?;
    let iso_date = format!("{}", datetime.format("%Y-%m-%d %H:%M:%S"));
    let delta = format!(
        "{}",
        humantime::format_duration(std::time::Duration::from_secs(
            (Utc::now().timestamp() - ts) as u64
        ))
    );
    Ok((iso_date, delta))
}

pub fn console_dump<T>(outbound_array: Option<Vec<T>>)
where
    T: Display,
{
    if let Some(vec) = outbound_array {
        for x in vec {
            safe_println(&format!("{}", x));
        }
    }
}
