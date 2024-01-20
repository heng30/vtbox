use anyhow::{Context, Result};
use chrono::{DateTime, Local, Utc};

pub fn local_now(format: &str) -> String {
    return Local::now().format(format).to_string();
}

pub fn from_timestamp(timestamp: i64, format: &str) -> Result<String> {
    let dt: DateTime<Utc> = DateTime::<Utc>::from_timestamp(timestamp, 0)
        .with_context(|| format!("invalid timestamp: {timestamp}"))?;
    Ok(dt.format(format).to_string())
}
