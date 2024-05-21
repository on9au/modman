use colored::Colorize;

use crate::{commands::command_structs::CommandOptions, errors::ModManError};

pub fn command_add_source(options: &CommandOptions) -> Result<(), ModManError> {
    /* 
        The arguments are as follows for 'install' add-source:
        <source_URL>      - The URL of the source.

        This argument can be repeated as much times as possible to add multiple sources at a time.

        Flags:
        --do-not-validate - Prevents WinForge from validating the URL (a.k.a. ensuring the URL is actually a WinForge source server.)
    */

    if options.parameters.is_empty() {
        return Err(ModManError::NoArguments);
    }

    let do_not_validate = options.flags.contains(&"--do-not-validate".to_string());

    if do_not_validate {
        println!("{}", "'--do-not-validate' tag detected. Not validating sources...".yellow().italic());
    }

    let mut sources: Vec<String> = Vec::with_capacity(options.parameters.len());

    for param in &options.parameters {
        if !do_not_validate {
            match validate_source(param) {
                Ok(()) => sources.push(param.clone()),
                Err(e) => println!("{} '{}': {}", "Could not validate source".yellow(), param, e),
            }
        } else {
            sources.push(param.clone());
        }
    }

    if sources.is_empty() {
        return Err(ModManError::NoValidSources);
    }

    println!("{}", "Adding sources:".green().bold());
    for source in &sources {
        println!("{}", source);
    }

    println!("{}: Duplicate sources will be ignored.", "Note".yellow());

    match crate::config::write_sources_to_config(&sources) {
        Ok(()) => Ok(()),
        Err(e) => return Err(e)
    }
}

fn validate_source(source: &str) -> Result<(), String> {
    // Implement your actual validation logic here
    if source.is_empty() {
        return Err("Source URL is empty".to_string());
    }
    if !source.starts_with("https://") {
        return Err("Source URL must start with 'https://'".to_string());
    }
    // Add more validation methods, like calling an API request to ensure that it responds appropriately.
    Ok(())
}