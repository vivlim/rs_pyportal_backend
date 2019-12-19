use config::{ConfigError, Config, File};
use serde::{Deserialize};

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub trello_key: String,
    pub trello_token: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        s.merge(File::with_name("Settings"))?;

        s.try_into()
    }
}