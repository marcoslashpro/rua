use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RuaConfiguration {
    pub model: RuaConfigurationModel,
    pub base_url: RuaConfigurationBaseUrl,
}

impl RuaConfiguration {
    pub fn new() -> Result<Self, toml::de::Error> {
        let configuration_content = fs::read_to_string(PathBuf::from("../rua/configuration.toml"))
            .expect("Failed to read configuration.toml. Make sure that the file exists.");
        let configuration: RuaConfiguration = toml::from_str(configuration_content.as_str())?;

        Ok(configuration)
    }
}

pub type RuaConfigurationModel = String;
pub type RuaConfigurationBaseUrl = String;