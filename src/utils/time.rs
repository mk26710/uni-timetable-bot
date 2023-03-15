use anyhow::{anyhow, Result};
use chrono::{DateTime, FixedOffset, Utc};

pub const TIME_OFFSET_SECONDS: i32 = 3600 * 4;

pub fn global_offset() -> Result<FixedOffset> {
    FixedOffset::east_opt(TIME_OFFSET_SECONDS).ok_or(anyhow!("seconds are out of bounds"))
}

pub fn now() -> Result<DateTime<FixedOffset>> {
    let offset = global_offset()?;
    Ok(Utc::now().with_timezone(&offset))
}
