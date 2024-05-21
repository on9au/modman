use crate::{commands::command_structs::CommandOptions, errors::ModManError};

#[derive(Debug)]
struct Package {
    search_term: String,
    source: crate::datatypes::ModSources
}

impl Package {
    // A constructor to create a new Package instance.
    fn new(search_term: String, source: Option<&str>) -> Result<Self, String> {
        let source: crate::datatypes::ModSources = match source {
            Some(s) => s.parse::<crate::datatypes::ModSources>()?,
            None => crate::datatypes::ModSources::Modrinth, // Default to Modrinth if no source is provided
        };
        Ok(Package { search_term, source })
    }
}

pub fn command_add(options: &CommandOptions) -> Result<(), ModManError> {
    /* 
        The arguments are as follows for 'add' command:
        <modrinth / curseforge (optional)>@<package_slug / package_ID>
        <modrinth / curseforge (optional)>@     - The source to look through only, if specified.
        <package_slug / package_ID>             - The name of the package being installed.

        ModMan prioritizes modrinth over curseforge. Therefore, if source is left blank (text before @), then modrinth is used.

        This argument can be repeated as much times as possible to install multiple mods at a time.
    */
    if options.parameters.is_empty() {
        return Err(ModManError::NoArguments);
    }

    // Define a vec of packages to be added.
    let mut packages: Vec<Package> = Vec::with_capacity(options.parameters.len());

    // For each argument (mod/package), parse into Package struct.
    for arg in &options.parameters {
        // Find the position of '@' in the argument
        if let Some(at_pos) = arg.find('@') {
            let source = &arg[..at_pos];
            let search_term = &arg[at_pos + 1..];
            
            if search_term.is_empty() {
                return Err(ModManError::InvalidCommandArguments(search_term.to_string()));
            }
            if source.is_empty() {
                return Err(ModManError::InvalidCommandArguments(search_term.to_string()));
            }
            packages.push(match Package::new(search_term.to_string(), Some(source)) {
                Ok(result) => result,
                Err(_) => return Err(ModManError::InvalidCommandArguments(search_term.to_string()))
            });
        } else {
            if arg.is_empty() {
                return Err(ModManError::InvalidCommandArguments(arg.to_string()));
            }
            packages.push(match Package::new(arg.to_string(), None) {
                Ok(result) => result,
                Err(_) => return Err(ModManError::InvalidCommandArguments(arg.to_string()))
            });
        }
    }


    Ok(())
}