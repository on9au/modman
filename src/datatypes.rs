use std::fmt;

use serde::{Deserialize, Serialize};

// Config File struct
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub game_version: String,
    pub game_loader: String,
    pub allowed_release_types: Vec<ReleaseTypes>,
    pub mods_folder: std::path::PathBuf,
    pub mods: Vec<Mod>,
}

// Mods struct
#[derive(Debug, Serialize, Deserialize)]
pub struct Mod {
    pub source: ModSources,
    pub id: String,
    pub name: String,
}

// Mod sources
#[derive(Debug, Serialize, Deserialize)]
pub enum ModSources {
    Modrinth,
    CurseForge,
}

impl fmt::Display for ModSources {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ModSources::Modrinth => write!(f, "Modrinth"),
            ModSources::CurseForge => write!(f, "CurseForge"),
        }
    }
}

impl std::str::FromStr for ModSources {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "modrinth" => Ok(ModSources::Modrinth),
            "curseforge" => Ok(ModSources::CurseForge),
            _ => Err(format!("Invalid source: {}", s)),
        }
    }
}

// Release types enums
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum ReleaseTypes {
    Release,
    Beta,
    Alpha,
}

impl fmt::Display for ReleaseTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            ReleaseTypes::Release => write!(f, "release"),
            ReleaseTypes::Beta => write!(f, "beta"),
            ReleaseTypes::Alpha => write!(f, "alpha"),
        }
    }
}

pub fn format_release_types(release_types: &[ReleaseTypes]) -> String {
    let mut formatted = String::new();
    for (index, release_type) in release_types.iter().enumerate() {
        if index > 0 {
            formatted.push_str(", ");
        }
        formatted.push_str(&release_type.to_string());
    }
    formatted
}