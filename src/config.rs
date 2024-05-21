use std::{fs, path::PathBuf};

use crate::{errors::ModManError, structs::Config};

pub fn save_config(dir: &PathBuf, config: &Config) -> Result<(), ModManError> {
    let config_path = dir.join("config.toml");
    let config_data = toml::to_string_pretty(config).map_err(|e| ModManError::SerializationError(e))?;
    fs::write(config_path, config_data).map_err(|e| ModManError::IoError(e))
}

pub fn read_config(dir: &PathBuf) -> Result<Config, ModManError> {
    let config_path = dir.join("config.toml");

    if !config_path.exists() {
        return Err(ModManError::FileNotFound);
    }
    
    let toml_content = fs::read_to_string(config_path)
        .map_err(|e| ModManError::IoError(e))?;

    let config: Config = toml::from_str(&toml_content)
        .map_err(|e| ModManError::DeserializationError(e))?;

    Ok(config)
}