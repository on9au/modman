use std::collections::HashSet;
use std::io::{self, Write};

use colored::Colorize;
use crossterm::cursor::MoveToPreviousLine;
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};

use crate::errors::ModManError;
use crate::structs::Config;
use crate::utils::get_current_working_dir;

pub fn command_init() -> Result<(), ModManError> {
    let mut game_version = String::new();
    let mut game_loader = String::new();
    let mut allowed_release_types = String::new();
    let mut mods_folder = String::new();

    let allowed_values: HashSet<&str> = ["release", "beta", "alpha"].iter().cloned().collect();

    let current_dir = match get_current_working_dir() {
        Ok(path) => path,
        Err(e) => return Err(ModManError::IoError(e))
    };

    println!(" {} {}: {}", "i".cyan().bold(), "Current directory".bold(), current_dir.display().to_string().bright_black());

    match crate::config::read_config(&current_dir) {
        Ok(_) => {
            println!(" {} {} {} {}", "!".red().bold(),
                "Found existing modman.toml file! 'modman init' will".red().bold(),
                "ERASE".red().bold().underline(), "modman.toml, thus removing the mods list!".red().bold()
            );
            println!("   | {}", "To prevent this, press '^C' (Ctrl + C) to exit.".red());
        },
        Err(ModManError::FileNotFound) => {},
        Err(e) => return Err(e)
    }

    // Ask user for version
    loop {
        print!(" {} {} {} ", "?".yellow().bold(), "Version of Minecraft".bold(), ">".bright_black());
        io::stdout().flush().unwrap();
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
        print!(" {} {} {} ", "?".yellow().bold(), "Loader of Minecraft".bold(), "[fabric, quilt, forge] >".bright_black());
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut game_loader).unwrap();
        game_loader = game_loader.trim().to_owned();

        if confirm_input(&game_loader) {
            break;
        } else {
            game_loader = String::new();
        }
    }

    // Ask user for default allowed release types
    loop {
        print!(" {} {} {} ", "?".yellow().bold(), "Default Allowed Release Types (alpha, beta, release) (seperated by comma)".bold(), "[Default: 'alpha, beta, release'] >".bright_black());
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut allowed_release_types).unwrap();
        allowed_release_types = allowed_release_types.trim().to_owned();

        if allowed_release_types.is_empty() {
            allowed_release_types = "alpha, beta, release".to_string()
        }

        let input_values: HashSet<&str> = allowed_release_types.split(',').map(|s| s.trim()).collect();
        let invalid_values: HashSet<_> = input_values.difference(&allowed_values).collect();
        let repeated_values: HashSet<_> = input_values.iter().filter(|&v| input_values.iter().filter(|&&x| x == *v).count() > 1).collect();

        if invalid_values.is_empty() && repeated_values.is_empty() {
            if confirm_input(&allowed_release_types) {
                break;
            } else {
                allowed_release_types.clear();
            }
        } else {
            println!(" {} Invalid or repeated values detected. Please re-enter the default allowed release types.", "!".red().bold());
            allowed_release_types.clear();
        }
    }

    // Ask user for mods folder
    loop {
        print!(" {} {} {} ", "?".yellow().bold(), "Mods Folder".bold(), "[Default is './mods'] >".bright_black());
        io::stdout().flush().unwrap();
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

    println!("{} {}", "OK".green().bold(), "Saving configuration. These settings will be used when you run modman in this directory.".bold());
    println!("   {} {} {} {}", "To reset configuration, run".bright_black(), "'modman init'".bright_black().bold(), "or modify".bright_black(), "config.toml".bright_black().bold());
    let config = Config {
        game_loader,
        game_version,
        allowed_release_types: allowed_release_types.split(',').map(|s| s.trim().to_owned()).collect(),
        mods_folder: std::path::PathBuf::from(mods_folder),
        mods: Vec::new(), // Empty mods array for now
    };
    crate::config::save_config(&current_dir, &config)?;

    Ok(())
}

fn confirm_input(input: &str) -> bool {
    // Ask for confirmation
    print!("   | {}{}{}{} ", "You entered: '".bold(), input.yellow().bold(), "'. Is this correct? ".bold(), "[y/N] >".bright_black());
    io::stdout().flush().unwrap();

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