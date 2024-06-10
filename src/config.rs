use std::{fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
    datatypes::{Config, LockMod},
    errors::ModManError,
};

// modman.toml

pub fn save_config(dir: &Path, config: &Config) -> Result<(), ModManError> {
    let config_path = dir.join("modman.toml");
    let config_data = toml::to_string_pretty(config).map_err(ModManError::SerializationError)?;
    fs::write(config_path, config_data).map_err(ModManError::IoError)
}

pub fn read_config(dir: &Path) -> Result<Config, ModManError> {
    let config_path = dir.join("modman.toml");

    if !config_path.exists() {
        return Err(ModManError::FileNotFound);
    }

    let toml_content = fs::read_to_string(config_path).map_err(ModManError::IoError)?;

    if toml_content.is_empty() {
        return Err(ModManError::FileIsEmpty);
    }

    let config: Config =
        toml::from_str(&toml_content).map_err(ModManError::DeserializationError)?;

    Ok(config)
}

// modman.lock

// This container solves toml serialization and deserialization errors.
#[derive(Serialize, Deserialize)]
struct LockModContainer {
    lockmod: Vec<LockMod>,
}

pub fn save_lockfile(dir: &Path, lockmod: &Vec<LockMod>) -> Result<(), ModManError> {
    let lockfile_path = dir.join("modman.lock");
    let lockmod_container = LockModContainer {
        lockmod: lockmod.to_owned(),
    };
    let lockfile_data =
        toml::to_string_pretty(&lockmod_container).map_err(ModManError::SerializationError)?;
    fs::write(lockfile_path, lockfile_data).map_err(ModManError::IoError)
}

pub fn read_lockfile(dir: &Path) -> Result<Vec<LockMod>, ModManError> {
    let lockfile_path = dir.join("modman.lock");

    if !lockfile_path.exists() {
        return Err(ModManError::FileNotFound);
    }

    let toml_content = fs::read_to_string(lockfile_path).map_err(ModManError::IoError)?;

    if toml_content.is_empty() {
        return Err(ModManError::FileIsEmpty);
    }

    let lockmod_container: LockModContainer =
        toml::from_str(&toml_content).map_err(ModManError::DeserializationError)?;

    let lockmod = lockmod_container.lockmod;

    Ok(lockmod)
}
