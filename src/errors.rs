#[derive(Debug)]
pub enum ModManError {
    CommandNotFound,
    InvalidCommandArguments(String),
    IoError(std::io::Error),
    NoVersionAfterAt(String),
    NoArguments,
    NoValidSources,
}

impl std::fmt::Display for ModManError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModManError::CommandNotFound => write!(f, "Command not found."),
            ModManError::InvalidCommandArguments(msg) => write!(f, "Invalid command arguments: {}", msg),
            ModManError::IoError(err) => write!(f, "IO error: {}", err),
            ModManError::NoVersionAfterAt(package) => write!(f, "No version specified for package: {}", package),
            ModManError::NoArguments => write!(f, "No arguments passed."),
            ModManError::NoValidSources => write!(f, "No valid sources specified."),
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
            ModManError::NoVersionAfterAt(_) => 4,
            ModManError::NoArguments => 5,
            ModManError::NoValidSources => 6,
        }
    }
}