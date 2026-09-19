use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Configuration {
    pub model: ConfigurationModel,
    pub base_url: ConfigurationBaseUrl,
}

impl Configuration {
    pub fn new() -> Result<Self, toml::de::Error> {
        let configuration_content = fs::read_to_string(PathBuf::from("../rua/configuration.toml"))
            .expect("Failed to read configuration.toml. Make sure that the file exists.");
        let configuration: Configuration = toml::from_str(configuration_content.as_str())?;

        Ok(configuration)
    }
}

pub type ConfigurationModel = String;
pub type ConfigurationBaseUrl = String;