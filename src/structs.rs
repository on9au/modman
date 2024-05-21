use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub game_version: String,
    pub game_loader: String,
    pub allowed_release_types: Vec<String>,
    pub mods_folder: std::path::PathBuf,
    pub mods: Vec<String>,
}