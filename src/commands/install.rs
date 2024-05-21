use crate::{commands::command_structs::CommandOptions, errors::ModManError};

#[derive(Debug)]
struct Package {
    name: String,
    version: String,
}

impl Package {
    // A constructor to create a new Package instance.
    fn new(name: String, version: Option<String>) -> Self {
        // If version is provided, use version, or default to "latest" 
        let version = version.unwrap_or_else(|| "latest".to_string());
        Package { name, version }
    }
}

pub fn command_install(options: &CommandOptions) -> Result<(), ModManError> {
    /* 
        The arguments are as follows for 'install' command:
        <package_name>@<version_number>
        <package_name>    - The name of the package being installed.
        @<version_number> - The version of the package being installed, followed by an '@' symbol to indicate version.

        This argument can be repeated as much times as possible to install multiple packages at a time.
    */
    if options.parameters.is_empty() {
        return Err(ModManError::NoArguments);
    }

    let mut packages: Vec<Package> = Vec::with_capacity(options.parameters.len());

    for arg in &options.parameters {
        // Find the position of '@' in the argument
        if let Some(at_pos) = arg.find('@') {
            let package_name = &arg[..at_pos];
            let version = &arg[at_pos + 1..];
            
            if package_name.is_empty() {
                return Err(ModManError::InvalidCommandArguments(package_name.to_string()));
            }
            if version.is_empty() {
                return Err(ModManError::NoVersionAfterAt(package_name.to_string()));
            }
            
            packages.push(Package::new(package_name.to_string(), Some(version.to_string())));
        } else {
            if arg.is_empty() {
                return Err(ModManError::InvalidCommandArguments(arg.to_string()));
            }
            packages.push(Package::new(arg.to_string(), None));
        }
    }


    Ok(())
}