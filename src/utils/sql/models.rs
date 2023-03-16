use anyhow::{anyhow, Result};
use sqlx::PgExecutor;

use super::types::UserEntry;

pub async fn get_user_by_id_opt(
    executor: impl PgExecutor<'_>,
    id: i64,
) -> Result<Option<UserEntry>> {
    let sql = r#"SELECT * FROM users WHERE id = $1;"#;

    let entry = sqlx::query_as::<_, UserEntry>(sql)
        .bind(id)
        .fetch_optional(executor)
        .await?;

    Ok(entry)
}

pub async fn get_user_entry_by_id(executor: impl PgExecutor<'_>, id: i64) -> Result<UserEntry> {
    let entry_opt = get_user_by_id_opt(executor, id).await?;

    entry_opt.ok_or(anyhow!("user entry not found"))
}
