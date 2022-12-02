use std::collections::HashSet;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub auth_users: HashSet<String>,
    pub key: String,
    pub db_uri: String,
}

pub fn get_config(input: &str) -> Result<Config, toml::de::Error> {
    toml::from_str(input)
}
