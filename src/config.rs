use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use serde::ser::Error;

#[derive(Debug, Deserialize, Serialize)]
pub struct WindowConfig {
    pub title: Option<String>,
    pub executable: Option<String>,
    pub opacity: u8,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub default_opacity: Option<u8>,
    pub specific_windows: Vec<WindowConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            default_opacity: None,
            specific_windows: Vec::new(),
        }
    }
}

pub fn load_config(path: &str) -> Result<Config, serde_yaml::Error> {
    if !Path::new(path).exists() {
        return Ok(Config::default());
    }
    let contents = fs::read_to_string(path).unwrap_or_default();
    serde_yaml::from_str(&contents)
}

pub fn save_config(config: &Config, path: &str) -> Result<(), serde_yaml::Error> {
    let yaml = serde_yaml::to_string(config)?;
    fs::write(path, yaml).map_err(|_| serde_yaml::Error::custom("Failed to write config file"))?;
    Ok(())
}

