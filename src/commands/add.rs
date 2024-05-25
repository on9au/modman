use std::{pin::Pin, sync::Arc};

use reqwest::Client;

use colored::Colorize;

use crate::{
    actionheader, alert, api::modrinth::fetch_modrinth_mod, commands::command_structs::CommandOptions, confirm, datatypes::{DependencyType, GameLoader, LockDependency, LockMod, ModSources}, errors::ModManError, info, install::download_all_mods, request, utils::convert_lock_mods_to_tuples, APP_USER_AGENT
};

use std::io::{self, Write};

#[derive(Debug)]
struct Package {
    search_term: String,
    source: ModSources
}

impl Package {
    // A constructor to create a new Package instance.
    fn new(search_term: String, source: Option<&str>) -> Result<Self, String> {
        let source: ModSources = match source {
            Some(s) => s.parse::<crate::datatypes::ModSources>()?,
            None => ModSources::Modrinth, // Default to Modrinth if no source is provided
        };
        Ok(Package { search_term, source })
    }
}

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

    let config = match crate::config::read_config(&current_directory) {
        Ok(result) => result,
        Err(ModManError::FileNotFound) => {
            alert!("No config file (modman.toml) found for this directory!");
            alert!("Please run 'modman init' to generate a config file.");
            return Err(ModManError::FileNotFound)
        },
        Err(ModManError::DeserializationError(e)) => {
            alert!("Either config file modman.toml has incorrect information, or is corrupt. Please modify modman.toml, or");
            alert!("delete it to reset the configuration.");
            return Err(ModManError::DeserializationError(e))
        }
        Err(e) => return Err(e)
    };

    info!("Found configuration file for this directory.");
    info!("Using version:", config.game_version.to_string());
    info!("Using loader: ", config.game_loader.to_string());

    print!("\n");
    actionheader!("Fetching Mod(s)");

    // Define a vec of packages to be added.
    let mut packages: Vec<Package> = Vec::with_capacity(options.parameters.len());

    // Define a vec of matches to be added. Added when interacting with Modrinth/CurseForge API
    let mut mod_matches = Vec::new();
    let mut mods_to_install: Vec<LockMod> = Vec::new();

    // For each argument (mod/package), parse into Package struct.
    for arg in &options.parameters {
        // Find the position of '@' in the argument
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
                // Async fetching.
                tokio::spawn(async move {
                    match fetch_modrinth_mod(&client, &search_term, &game_version, &game_loader).await {
                        Ok(result) => Ok(result),
                        Err(err) => Err(ModManError::CannotFindMod(format!("{}", err))),
                    }
                })
            }
            ModSources::CurseForge => unimplemented!(), // Handle CurseForge fetch here
        };
        mod_matches.push(mod_match);
    }

    let results: Vec<Result<LockMod, ModManError>> = futures::future::join_all(mod_matches).await.into_iter().map(|res| {
        res.unwrap_or_else(|join_error| Err(ModManError::APIFetchError(format!("Task failed: {:?}", join_error))))
    }).collect();
    // Handle the results
    for result in results {
        match result {
            Ok(mod_result) => {
                let message = "Found mod:             '".to_string() + &mod_result.name + "'";
                confirm!(message);
                if !mods_to_install.iter().any(|m| m.id == mod_result.id) {
                    mods_to_install.push(mod_result.clone());
                    if ignore_dependencies {
                        break;
                    }
                    match handle_dependencies(&client, &mut mods_to_install, &mod_result.dependencies, &config.game_version, &config.game_loader).await {
                        Ok(_) => {},
                        Err(e) => {
                            let message = "Cannot find mod:       '".to_string() + &e.to_string() + "'";
                            alert!(message)
                        },
                    };
                }
            }
            Err(e) => alert!(e.to_string()),
        }
    }

    if mods_to_install.is_empty() {
        return Err(ModManError::NoMods("get".to_owned()));
    }

    // Transaction prompt:
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

    let tuples = convert_lock_mods_to_tuples(&config, mods_to_install);
    match download_all_mods(&client, tuples).await {
        Ok(_) => {},
        Err(e) => return Err(ModManError::TransactionDownloadError(e)),
    };


    Ok(())
}

async fn handle_dependencies(
    client: &Client,
    mods_to_install: &mut Vec<LockMod>,
    dependencies: &[LockDependency],
    minecraft_version: &String,
    loader: &GameLoader,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = Vec::new();

    for dep in dependencies {
        if !mods_to_install.iter().any(|m| m.id == dep.project_id) {
            match dep.dependency_type {
                DependencyType::Required => {
                    let client = client.clone();
                    let dep_id = dep.project_id.clone();
                    let minecraft_version = minecraft_version.clone();
                    let loader = loader.clone();

                    tasks.push(tokio::spawn(async move {
                        fetch_modrinth_mod(&client, &dep_id, &minecraft_version, &loader).await
                    }));
                }
                DependencyType::Optional => {
                    // Optional dependencies can be skipped or handled differently if needed
                }
                DependencyType::Incompatible => {
                    return Err(Box::new(ModManError::IncompatibleDependency(format!(
                        "Incompatible mod: {}",
                        dep.project_id
                    ).into())));
                }
                DependencyType::Embedded => {
                    // Handle embedded dependencies if needed
                }
            }
        }
    }

    let results = futures::future::join_all(tasks).await;
    for result in results {
        match result {
            Ok(Ok(dep_mod)) => {
                if !mods_to_install.iter().any(|m| m.id == dep_mod.id) {
                    mods_to_install.push(dep_mod.clone());
                    let fut = handle_dependencies(client, mods_to_install, &dep_mod.dependencies, minecraft_version, loader);
                    Pin::from(Box::new(fut)).await?;
                }
            }
            Ok(Err(e)) => {
                return Err(e);
            }
            Err(join_error) => {
                return Err(Box::new(ModManError::APIFetchError(format!(
                    "Task failed: {:?}",
                    join_error
                ))));
            }
        }
    }
    Ok(())
}

fn calculate_total_size(mods_to_install: &[LockMod]) -> String {
    let total_size: u64 = mods_to_install.iter().map(|mod_| mod_.size).sum();
    let mut total_size_str = String::new();

    if total_size < 1024 {
        total_size_str.push_str(&format!("{} B", total_size));
    } else if total_size < 1024 * 1024 {
        total_size_str.push_str(&format!("{:.2} KB", total_size as f64 / 1024.0));
    } else if total_size < 1024 * 1024 * 1024 {
        total_size_str.push_str(&format!("{:.2} MB", total_size as f64 / (1024.0 * 1024.0)));
    } else {
        total_size_str.push_str(&format!("{:.2} GB", total_size as f64 / (1024.0 * 1024.0 * 1024.0)));
    }

    total_size_str
}