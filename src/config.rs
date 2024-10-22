use serde::Deserialize;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Deserialize)]
pub struct WindowConfig {
    pub title: Option<String>,
    pub executable: Option<String>,
    pub opacity: u8,
}
#[derive(Debug, Deserialize)]
pub struct Config {
    pub default_opacity: Option<u8>,
    pub specific_windows: Vec<WindowConfig>,
}
pub fn load_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let config: Config = serde_yaml::from_str(&contents)?;
    Ok(config)
}
