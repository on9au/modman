use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

// Config File struct
#[derive(Serialize, Deserialize)]
pub struct Config {
    pub game_version: String,
    pub game_loader: GameLoader,
    pub allowed_release_types: Vec<ReleaseTypes>,
    pub mods_folder: std::path::PathBuf,
    pub mods: Vec<Mod>,
}

// Game loader enums
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GameLoader {
    Bukkit,
    BungeeCord,
    Canvas,
    Datapack,
    Fabric,
    Folia,
    Forge,
    Iris,
    LiteLoader,
    Minecraft,
    ModLoader,
    NeoForge,
    OptiFine,
    Paper,
    Purpur,
    Quilt,
    Rift,
    Spigot,
    Sponge,
    Vanilla,
    Velocity,
    Waterfall,
}


impl fmt::Display for GameLoader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameLoader::Bukkit => write!(f, "bukkit"),
            GameLoader::BungeeCord => write!(f, "bungeecord"),
            GameLoader::Canvas => write!(f, "canvas"),
            GameLoader::Datapack => write!(f, "datapack"),
            GameLoader::Fabric => write!(f, "fabric"),
            GameLoader::Folia => write!(f, "folia"),
            GameLoader::Forge => write!(f, "forge"),
            GameLoader::Iris => write!(f, "iris"),
            GameLoader::LiteLoader => write!(f, "liteloader"),
            GameLoader::Minecraft => write!(f, "minecraft"),
            GameLoader::ModLoader => write!(f, "modloader"),
            GameLoader::NeoForge => write!(f, "neoforge"),
            GameLoader::OptiFine => write!(f, "optifine"),
            GameLoader::Paper => write!(f, "paper"),
            GameLoader::Purpur => write!(f, "purpur"),
            GameLoader::Quilt => write!(f, "quilt"),
            GameLoader::Rift => write!(f, "rift"),
            GameLoader::Spigot => write!(f, "spigot"),
            GameLoader::Sponge => write!(f, "sponge"),
            GameLoader::Vanilla => write!(f, "vanilla"),
            GameLoader::Velocity => write!(f, "velocity"),
            GameLoader::Waterfall => write!(f, "waterfall"),
        }
    }
}

impl std::str::FromStr for GameLoader {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bukkit" => Ok(GameLoader::Bukkit),
            "bungeecord" => Ok(GameLoader::BungeeCord),
            "canvas" => Ok(GameLoader::Canvas),
            "datapack" => Ok(GameLoader::Datapack),
            "fabric" => Ok(GameLoader::Fabric),
            "folia" => Ok(GameLoader::Folia),
            "forge" => Ok(GameLoader::Forge),
            "iris" => Ok(GameLoader::Iris),
            "liteloader" => Ok(GameLoader::LiteLoader),
            "minecraft" => Ok(GameLoader::Minecraft),
            "modloader" => Ok(GameLoader::ModLoader),
            "neoforge" => Ok(GameLoader::NeoForge),
            "optifine" => Ok(GameLoader::OptiFine),
            "paper" => Ok(GameLoader::Paper),
            "purpur" => Ok(GameLoader::Purpur),
            "quilt" => Ok(GameLoader::Quilt),
            "rift" => Ok(GameLoader::Rift),
            "spigot" => Ok(GameLoader::Spigot),
            "sponge" => Ok(GameLoader::Sponge),
            "vanilla" => Ok(GameLoader::Vanilla),
            "velocity" => Ok(GameLoader::Velocity),
            "waterfall" => Ok(GameLoader::Waterfall),
            _ => Err(format!("Invalid game launcher: {}", s)),
        }
    }
}


// Mods struct
#[derive(Debug, Serialize, Deserialize)]
pub struct Mod {
    pub source: ModSources,
    pub id: String,
    pub name: String,
}

// Mod sources
#[derive(Debug, Serialize, Deserialize, Clone)]
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

// Lockfile:
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LockMod {
    pub name: String,
    pub source: ModSources,
    pub id: String,
    pub version: String,
    pub file_name: String,
    pub release_date: String,
    pub sha512: String,
    pub download_url: String,
    pub dependencies: Vec<LockDependency>,
    pub size: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LockDependency {
    pub source: ModSources,
    pub project_id: String,
    pub dependency_type: DependencyType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DependencyType {
    Required,
    Optional,
    Incompatible,
    Embedded,
}

impl std::str::FromStr for DependencyType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "required" => Ok(DependencyType::Required),
            "optional" => Ok(DependencyType::Optional),
            "incompatible" => Ok(DependencyType::Incompatible),
            "embedded" => Ok(DependencyType::Embedded),
            _ => Err(format!("Invalid Dependency Type: {}", s)),
        }
    }
}

impl fmt::Display for DependencyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DependencyType::Required => write!(f, "required"),
            DependencyType::Optional => write!(f, "optional"),
            DependencyType::Incompatible => write!(f, "incompatible"),
            DependencyType::Embedded => write!(f, "embedded"),
        }
    }
}

impl TryFrom<crate::api::modrinth::ModrinthDependency> for LockDependency {
    type Error = String;

    fn try_from(dep: crate::api::modrinth::ModrinthDependency) -> Result<Self, Self::Error> {
        Ok(LockDependency {
            project_id: dep.project_id,
            dependency_type: DependencyType::from_str(&dep.dependency_type)?,
            source: ModSources::Modrinth,
        })
    }
}
