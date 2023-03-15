use std::sync::Arc;

use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

#[derive(Debug, Clone)]
pub struct Database {
    pub pool: Arc<Pool<Postgres>>,
}

impl Database {
    pub fn new(pool: Arc<Pool<Postgres>>) -> Self {
        Self { pool }
    }

    pub async fn create_pool(protocol: &str) -> Result<Pool<Postgres>> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(protocol)
            .await?;

        Ok(pool)
    }
}
