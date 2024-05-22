#[derive(Debug)]
pub enum ModManError {
    CommandNotFound,
    InvalidCommandArguments(String),
    IoError(std::io::Error),
    NoArguments,
    SerializationError(toml::ser::Error),
    DeserializationError(toml::de::Error),
    FileNotFound,
    ReqwestError(reqwest::Error),
    APIFetchError(String),
    CannotFindMod(String),
}

impl std::fmt::Display for ModManError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModManError::CommandNotFound => write!(f, "Command not found."),
            ModManError::InvalidCommandArguments(msg) => write!(f, "Invalid command arguments: {}", msg),
            ModManError::IoError(err) => write!(f, "IO error: {}", err),
            ModManError::NoArguments => write!(f, "No arguments passed."),
            ModManError::SerializationError(err) => write!(f, "Serialization error: {}", err),
            ModManError::DeserializationError(err) => write!(f, "Deserialization error: {}", err),
            ModManError::FileNotFound => write!(f, "File not found."),
            ModManError::ReqwestError(err) => write!(f, "Error with API client (Reqwest): {}", err),
            ModManError::APIFetchError(err) => write!(f, "Error with API request: {}", err),
            ModManError::CannotFindMod(err) => write!(f, "Cannot find mod:       '{}'", err),
        }
    }
}

impl std::error::Error for ModManError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ModManError::IoError(err) => Some(err),
            _ => None,
        }
    }
}

impl ModManError {
    pub fn exit_code(&self) -> i32 {
        match self {
            ModManError::CommandNotFound => 1,
            ModManError::InvalidCommandArguments(_) => 2,
            ModManError::IoError(_) => 3,
            ModManError::NoArguments => 4,
            ModManError::SerializationError(_) => 5,
            ModManError::DeserializationError(_) => 6,
            ModManError::FileNotFound => 7,
            ModManError::ReqwestError(_) => 8,
            ModManError::APIFetchError(_) => 9,
            ModManError::CannotFindMod(_) => 10,
        }
    }
}