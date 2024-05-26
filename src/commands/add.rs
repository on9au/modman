use std::{collections::HashSet, sync::Arc};
use reqwest::Client;
use colored::Colorize;
use crate::{
    actionheader, alert, api::modrinth::fetch_modrinth_mod, commands::command_structs::CommandOptions,
    config::{read_lockfile, save_config, save_lockfile}, confirm, datatypes::{LockMod, Mod, ModSources},
    errors::ModManError, info, install::download_all_mods, request, utils::convert_lock_mods_to_tuples, APP_USER_AGENT
};
use crate::commands::add_tools::dependencies::handle_dependencies;
use crate::commands::add_tools::package::Package;
use crate::utils::calculate_total_size;
use std::io::{self, Write};

pub async fn command_add(options: &CommandOptions) -> Result<(), ModManError> {
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

    let ignore_dependencies = options.flags.contains(&"--ignore-dependencies".to_string());

    if ignore_dependencies {
        info!("'--ignore-dependencies' tag detected. Ignoring dependencies...");
    }

    let current_directory = match crate::utils::get_current_working_dir() {
        Ok(result) => result,
        Err(e) => return Err(ModManError::IoError(e)),
    };

    // Load configuration file
    let mut config = match crate::config::read_config(&current_directory) {
        Ok(result) => result,
        Err(ModManError::FileNotFound) => {
            alert!("No config file (modman.toml) found for this directory!");
            alert!("Please run 'modman init' to generate a config file.");
            return Err(ModManError::FileNotFound)
        },
        Err(ModManError::FileIsEmpty) => {
            alert!("Config file (modman.toml) is empty!");
            alert!("Please run 'modman init' to generate a config file.");
            return Err(ModManError::FileIsEmpty)
        },
        Err(ModManError::DeserializationError(e)) => {
            alert!("Either config file modman.toml has incorrect information, or is corrupt. Please modify modman.toml, or");
            alert!("delete it to reset the configuration.");
            return Err(ModManError::DeserializationError(e))
        }
        Err(e) => return Err(e)
    };

    // Load lockfile
    let mut current_lockfile: Vec<LockMod> = match read_lockfile(&current_directory) {
        Ok(result) => result,
        Err(ModManError::FileNotFound) => Vec::new(),
        Err(ModManError::FileIsEmpty) => Vec::new(),
        Err(e) => return Err(e),
    };

    info!("Found configuration file for this directory.");
    info!("Using version:", config.game_version.to_string());
    info!("Using loader: ", config.game_loader.to_string());

    print!("\n");
    actionheader!("Fetching Mod(s)");

    let mut packages: Vec<Package> = Vec::with_capacity(options.parameters.len());
    let mut mod_matches = Vec::new();
    let mut mods_to_install: Vec<LockMod> = Vec::new();

    for arg in &options.parameters {
        if let Some(at_pos) = arg.find('@') {
            let source = &arg[..at_pos];
            let search_term = &arg[at_pos + 1..];

            if search_term.is_empty() {
                return Err(ModManError::InvalidCommandArguments(arg.to_string()));
            }
            if source.is_empty() {
                return Err(ModManError::InvalidCommandArguments(source.to_string()));
            }
            packages.push(match Package::new(search_term.to_string(), Some(source)) {
                Ok(result) => result,
                Err(_) => return Err(ModManError::InvalidCommandArguments(arg.to_string()))
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

    let client = Arc::new(match Client::builder()
        .user_agent(APP_USER_AGENT)
        .build() {
            Ok(result) => result,
            Err(e) => return Err(ModManError::ReqwestError(e)),
        });

    for package in packages {
        let client = Arc::clone(&client);
        let mod_match: tokio::task::JoinHandle<Result<LockMod, ModManError>> = match package.source {
            ModSources::Modrinth => {
                let search_term = package.search_term.clone();
                let game_version = config.game_version.clone();
                let game_loader = config.game_loader.clone();
                tokio::spawn(async move {
                    match fetch_modrinth_mod(&client, &search_term, &game_version, &game_loader).await {
                        Ok(result) => Ok(result),
                        Err(err) => Err(ModManError::CannotFindMod(format!("{}", err))),
                    }
                })
            }
            ModSources::CurseForge => unimplemented!(),
        };
        mod_matches.push(mod_match);
    }

    let results: Vec<Result<LockMod, ModManError>> = futures::future::join_all(mod_matches).await.into_iter().map(|res| {
        res.unwrap_or_else(|join_error| Err(ModManError::APIFetchError(format!("Task failed: {:?}", join_error))))
    }).collect();

    let mut already_installed_mods: HashSet<String> = HashSet::new(); // Track already installed mods to avoid duplicates

    for result in results {
        match result {
            Ok(mod_result) => {
                let message = "Found mod:             '".to_string() + &mod_result.name + "'";
                confirm!(message);
                // Check if mod is not already installed and its dependencies are not already installed
                if !already_installed_mods.contains(&mod_result.id) && !current_lockfile.iter().any(|lock_mod| lock_mod.id == mod_result.id) {
                    mods_to_install.push(mod_result.clone());
                    if ignore_dependencies {
                        already_installed_mods.insert(mod_result.id.clone());
                    } else {
                        match handle_dependencies(&client, &mut mods_to_install, &mod_result.dependencies, &config.game_version, &config.game_loader).await {
                            Ok(_) => {
                                // Insert the mod and its dependencies into the set of already installed mods
                                already_installed_mods.insert(mod_result.id.clone());
                                for dep in &mod_result.dependencies {
                                    already_installed_mods.insert(dep.project_id.clone());
                                }
                            },
                            Err(e) => {
                                let message = "Cannot find mod: '".to_string() + &e.to_string() + "'";
                                alert!(message);
                            },
                        };
                    }
                }
            }
            Err(e) => alert!(e.to_string()),
        }
    }

    if mods_to_install.is_empty() {
        return Err(ModManError::NoMods("get".to_owned()));
    }

    print!("\n");
    actionheader!("Get Mod(s) Transaction");
    info!("Mods to be installed:");
    for mod_result in &mods_to_install {
        println!("    {}", mod_result.name);
    }
    print!("\n");
    info!("Total download size: ", calculate_total_size(&mods_to_install));
    request!("Begin download/transaction?", "[Y/n]");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input = input.trim().to_owned();
    if input == "n" || input == "no" {
        confirm!("Cancelled transaction. Exiting...");
        return Ok(());
    }
    print!("\n");
    actionheader!("Transaction");

    let tuples = convert_lock_mods_to_tuples(&config, mods_to_install.clone());
    match download_all_mods(&client, tuples).await {
        Ok(_) => {},
        Err(e) => return Err(ModManError::TransactionDownloadError(e)),
    };
    confirm!("Transaction finished. All fetched mods have been downloaded.");
    info!("Writing to config and lockfile...");

    current_lockfile.append(&mut mods_to_install.clone());
    match save_lockfile(&current_directory, &current_lockfile) {
        Ok(_) => confirm!("Lockfile saved successfully."),
        Err(e) => return Err(e),
    };

    for mod_match in mods_to_install {
        let mod_input: Mod = Mod {
            source: mod_match.source,
            id: mod_match.id,
            name: mod_match.name,
        };
        config.mods.push(mod_input);
    }
    match save_config(&current_directory, &config) {
        Ok(_) => confirm!("Config file saved successfully."),
        Err(e) => return Err(e),
    }

    Ok(())
}
