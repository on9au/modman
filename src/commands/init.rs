use std::collections::HashSet;
use std::io::{self, Write};
use std::str::FromStr;

use colored::Colorize;
use crossterm::cursor::MoveToPreviousLine;
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};

use crate::errors::ModManError;
use crate::datatypes::{Config, ReleaseTypes, GameLoader};
use crate::{alert, confirm, info, request, requestconfirm};
use crate::utils::get_current_working_dir;

pub fn command_init() -> Result<(), ModManError> {
    let mut game_version = String::new();
    let mut game_loader: Option<GameLoader> = None;
    let mut allowed_release_types: Vec<ReleaseTypes> = Vec::new();
    let mut mods_folder = String::new();

    let current_dir = match get_current_working_dir() {
        Ok(path) => path,
        Err(e) => return Err(ModManError::IoError(e))
    };

    info!("Current directory:", current_dir.display().to_string());

    match crate::config::read_config(&current_dir) {
        Ok(_) => {
            alert!("Found existing modman.toml file! 'modman init' will ERASE modman.toml, thus removing the mods list!");
            alert!("To prevent this, press '^C' (Ctrl + C) to exit.")
        },
        Err(ModManError::FileNotFound) => {},
        Err(ModManError::DeserializationError(e)) => {
            alert!("Either config file modman.toml has incorrect information, or is corrupt.");
            alert!("'modman init' will RESET the broken config file.");
            alert!("It might be a good idea to create a backup of the config file if you have mods saved there.");
            println!("   {} {}", "The error was:".bright_red(), e.to_string());
            info!("Continue below with modman init to reset broken config...");
        }
        Err(e) => return Err(e)
    }

    // Ask user for version
    loop {
        request!("Version of Minecraft", "[Any valid Minecraft version]");
        io::stdin().read_line(&mut game_version).unwrap();
        game_version = game_version.trim().to_owned();

        if confirm_input(&game_version) {
            break;
        } else {
            game_version = String::new();
        }
    }

    // Ask user for game_loader
    loop {
        request!("Loader of Minecraft", "[fabric, quilt, forge, etc.]");
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_owned().to_lowercase();

        let mut is_valid: bool = true;
        match GameLoader::from_str(&input) {
            Ok(result) => {
                game_loader = Some(result);
            },
            Err(_) => {
                println!(" {} Invalid value '{}' detected. Please enter a valid Minecraft loader.", "!".red().bold(), input.bold());
                is_valid = false;
            }
        };
        if is_valid {
            if confirm_input(&game_loader.clone().unwrap().to_string()) {
                break;
            }
        }
        input.clear();
    }

    // Ask user for default allowed release types
    loop {
        request!("Default Allowed Release Types (alpha, beta, release) (seperated by comma)", "[Default: 'alpha, beta, release']");
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input = input.trim().to_owned();

        if input.is_empty() {
            input = "alpha, beta, release".to_string()
        }

        let split_values: Vec<&str> = input.split(',').map(|s| s.trim()).collect();

        let mut is_valid = true;
        for value in split_values.iter() {
            match value.to_lowercase().as_str() {
                "release" => allowed_release_types.push(ReleaseTypes::Release),
                "beta" => allowed_release_types.push(ReleaseTypes::Beta),
                "alpha" => allowed_release_types.push(ReleaseTypes::Alpha),
                _ => {
                    println!(" {} Invalid value '{}' detected. Please enter only 'release', 'beta', or 'alpha'.", "!".red().bold(), value.bold());
                    is_valid = false;
                    allowed_release_types.clear();
                    break;
                }
            }
        }

        if is_valid {
            let unique_values: HashSet<_> = allowed_release_types.iter().collect();
            if unique_values.len() != allowed_release_types.len() {
                println!(" {} Repeated values detected. Please enter each release type only once.", "!".red().bold());
                allowed_release_types.clear();
            } else {
                if confirm_input(&crate::datatypes::format_release_types(&allowed_release_types)) {
                    break;
                } else {
                    allowed_release_types.clear();
                }
            }
        }
    }

    // Ask user for mods folder
    loop {
        request!("Mods Folder", "[Default is './mods']");
        io::stdin().read_line(&mut mods_folder).unwrap();
        mods_folder = mods_folder.trim().to_owned();

        if mods_folder.is_empty() {
            mods_folder = "./mods".to_string()
        }

        if confirm_input(&mods_folder) {
            break;
        } else {
            mods_folder = String::new();
        }
    }

    confirm!("Saving configuration. These settings will be used when you run modman in this directory.");
    
    let config = Config {
        game_loader: game_loader.expect("Game Loader variable was empty somehow during init command! Please report this issue."),
        game_version,
        allowed_release_types: allowed_release_types,
        mods_folder: std::path::PathBuf::from(mods_folder),
        mods: Vec::new(), // Empty mods array for now
    };
    crate::config::save_config(&current_dir, &config)?;

    Ok(())
}

fn confirm_input(input: &str) -> bool {
    // Ask for confirmation
    requestconfirm!("Is this correct:", input, "[y/N]");

    let mut confirmation = String::new();
    io::stdin().read_line(&mut confirmation).unwrap();
    let confirmation = confirmation.trim().to_lowercase();

    execute!(
        io::stdout(),
        MoveToPreviousLine(1),
        Clear(ClearType::CurrentLine)
    ).unwrap();

    if !(confirmation == "y") && !(confirmation == "yes") {
        execute!(
            io::stdout(),
            MoveToPreviousLine(1),
            Clear(ClearType::CurrentLine)
        ).unwrap();
    }

    confirmation == "y" || confirmation == "yes"
}