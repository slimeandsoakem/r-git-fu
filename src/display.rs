use crate::primitives::FuError;
use chrono::{DateTime, TimeZone, Utc};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::{ASCII_BORDERS_ONLY_CONDENSED, NOTHING};
use comfy_table::Table;

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
pub fn standard_table_setup(plain_tables: bool) -> Table {
    let mut table = Table::new();
    table
        .set_content_arrangement(comfy_table::ContentArrangement::Dynamic)
        .apply_modifier(UTF8_ROUND_CORNERS);
    let table_style = if plain_tables {
        NOTHING
    } else {
        ASCII_BORDERS_ONLY_CONDENSED
    };
    table.load_preset(table_style);
    table
}
