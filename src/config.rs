use std::{fs, path::PathBuf};

use crate::{datatypes::{Config, LockMod}, errors::ModManError};

// modman.toml

pub fn save_config(dir: &PathBuf, config: &Config) -> Result<(), ModManError> {
    let config_path = dir.join("modman.toml");
    let config_data = toml::to_string_pretty(config).map_err(|e| ModManError::SerializationError(e))?;
    fs::write(config_path, config_data).map_err(|e| ModManError::IoError(e))
}

pub fn read_config(dir: &PathBuf) -> Result<Config, ModManError> {
    let config_path = dir.join("modman.toml");

    if !config_path.exists() {
        return Err(ModManError::FileNotFound);
    }
    
    let toml_content = fs::read_to_string(config_path)
        .map_err(|e| ModManError::IoError(e))?;

    let config: Config = toml::from_str(&toml_content)
        .map_err(|e| ModManError::DeserializationError(e))?;

    Ok(config)
}

// modman.lock

pub fn save_lockfile(dir: &PathBuf, lockmod: &Vec<LockMod>) -> Result<(), ModManError> {
    let lockfile_path = dir.join("modman.lock");
    let lockfile_data = toml::to_string_pretty(lockmod).map_err(|e| ModManError::SerializationError(e))?;
    fs::write(lockfile_path, lockfile_data).map_err(|e| ModManError::IoError(e))
}

pub fn read_lockfile(dir: &PathBuf) -> Result<Vec<LockMod>, ModManError> {
    let lockfile_path = dir.join("modman.lock");

    if !lockfile_path.exists() {
        return Err(ModManError::FileNotFound);
    }
    
    let toml_content = fs::read_to_string(lockfile_path)
        .map_err(|e| ModManError::IoError(e))?;

    let lockmod: Vec<LockMod> = toml::from_str(&toml_content)
        .map_err(|e| ModManError::DeserializationError(e))?;

    Ok(lockmod)
}