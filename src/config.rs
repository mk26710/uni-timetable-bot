use serde::Deserialize;

use figment::{
    providers::{Format, Toml},
    Figment,
};

#[derive(Debug, Deserialize)]
pub struct Telegram {
    pub token: String,
    pub owner_ids: Vec<u64>,
}

#[derive(Debug, Deserialize)]
pub struct Database {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub telegram: Telegram,
    pub database: Database,
}

impl AppConfig {
    pub fn figment() -> AppConfig {
        Figment::new()
            .merge(Toml::file("./config.toml"))
            .extract()
            .unwrap()
    }
}
