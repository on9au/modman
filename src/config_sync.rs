use colored::Colorize;
use reqwest::Client;
use std::{collections::HashMap, fs, path::Path};

use crate::{
    alert,
    api::modrinth::modrinth_mod_from_hash,
    config::{read_lockfile, save_config, save_lockfile},
    datatypes::{DependencyType, LockDependency, LockMod, Mod, ModSources},
    errors::ModManError,
    install::calculate_sha512,
};

pub struct SyncFilesReturn {
    pub missing_dependencies: Vec<LockDependency>,
    pub new_mods: Vec<Mod>,
    pub to_reinstall_bad_checksum: Vec<LockMod>,
}

pub async fn sync_files(
    current_directory: &Path,
    client: &Client,
) -> Result<SyncFilesReturn, ModManError> {
    /*
        fn sync_files() compares installed mods, the lockfile, and the config file, then makes updates to synchronize all of these files.

        Step-by-Step Workflow:
            1. READ: Read config and lockfile.
            2. SCAN: Scan mods folder its checksum and filename.
            3. COMPARE: Verify its existence in the lockfile and config.
                New mod that doesn't exist in lockfile? Add it (maybe try to match before returning source: local ?)
                Lockfile entry that points to a non-existent mod in the folder? Remove it. If it is a dependency to any other mod, add to vec of missing dependencies.
            4. SYNC (modfiles to lockfile): (Try to) remove invalid entries, and add new entries (trying to match it to a mod with a source).
            5. SYNC (config to lockfile): Remove mods and its no longer used dependencies if config does not include the mod. Return list of mods to download if config file
            contains a mod that the lockfile doesn't.

        Returns:
            Vec of of missing mod IDs and source that other mods are dependent on.
            Vec of mod IDs and source to be fetched and resolved.
            Vec of lockmods that have incorrect checksums, and need to be re-installed.
    */

    let mut mod_files: Vec<(String, String)> = Vec::new(); // filename, sha512
    let mut to_reinstall_bad_checksum: Vec<LockMod> = Vec::new();
    let mut missing_dependencies: Vec<LockDependency> = Vec::new();
    let mut new_mods: Vec<Mod> = Vec::new();

    // (1) Read config and lockfile.
    // Load config
    let mut config = match crate::config::read_config(current_directory) {
        Ok(result) => result,
        Err(ModManError::FileNotFound) => {
            alert!("No config file (modman.toml) found for this directory!");
            alert!("Please run 'modman init' to generate a config file.");
            return Err(ModManError::FileNotFound);
        }
        Err(ModManError::FileIsEmpty) => {
            alert!("Config file (modman.toml) is empty!");
            alert!("Please run 'modman init' to generate a config file.");
            return Err(ModManError::FileIsEmpty);
        }
        Err(ModManError::DeserializationError(e)) => {
            alert!("Either config file modman.toml has incorrect information, or is corrupt. Please modify modman.toml, or");
            alert!("delete it to reset the configuration.");
            return Err(ModManError::DeserializationError(e));
        }
        Err(e) => return Err(e),
    };

    // Load lockfile
    let mut current_lockfile: Vec<LockMod> = match read_lockfile(current_directory) {
        Ok(result) => result,
        Err(ModManError::FileNotFound) => Vec::new(),
        Err(ModManError::FileIsEmpty) => Vec::new(),
        Err(e) => return Err(e),
    };

    // (2) Scan mods folder its checksum and filename.
    for entry in fs::read_dir(&config.mods_folder).map_err(ModManError::IoError)? {
        let entry = entry.map_err(ModManError::IoError)?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("jar") {
            let file_name = match path.file_name() {
                Some(result) => result,
                None => return Err(ModManError::FileNotFound),
            }
            .to_os_string()
            .into_string()
            .unwrap();
            let checksum = calculate_sha512(&path).map_err(ModManError::IoError)?;
            mod_files.push((file_name, checksum));
        }
    }

    // (3) Verify its existence in the lockfile and config.
    // (4) (modfiles to lockfile): (Try to) remove invalid entries, and add new entries (trying to match it to a mod with a source).
    // Match mod_files to lockfile (lockfile is temporarily as a hashmap here).
    let mut lockfile_map: HashMap<String, LockMod> = current_lockfile
        .iter()
        .cloned()
        .map(|m| (m.file_name.clone(), m.clone()))
        .collect();
    for (file_path, checksum) in &mod_files {
        let key = lockfile_map.get_key_value(file_path);
        // If mod is not found in lockfile, add it to lockfile and config.
        if key.is_none() {
            // Mod does not exist.
            // Try to match file to hash from source
            if let Ok(result) = modrinth_mod_from_hash(client, checksum).await {
                // Try Modrinth:
                fs::rename(file_path.clone(), result.clone().file_name)
                    .map_err(ModManError::IoError)?;
                current_lockfile.push(result.clone());

                config.mods.push(Mod {
                    source: result.source,
                    id: result.id,
                    name: result.name,
                });
                // TODO: Add implementation for curseforge.
            } else {
                // No matches to a source. Add as local instead...:
                current_lockfile.push(LockMod {
                    name: file_path.clone(),
                    source: ModSources::Local,
                    id: file_path.clone(),
                    version: "0".to_string(),
                    file_name: file_path.clone(),
                    release_date: "Unknown".to_string(),
                    sha512: checksum.clone(),
                    download_url: "Unknown".to_string(),
                    dependencies: vec![],
                    size: fs::metadata(file_path).map_err(ModManError::IoError)?.len(),
                });

                config.mods.push(Mod {
                    source: ModSources::Local,
                    id: file_path.clone(),
                    name: file_path.clone(),
                });
            }
            // Now that we added to the actual lockfile, we remove it from the map.
            lockfile_map.remove(file_path);
        } else if key.unwrap().1.sha512 == *checksum {
            // Mod already exists and checksum matches. Ignore and remove from map.
            lockfile_map.remove(file_path);
        } else {
            // Mod failed integrity check despite having same filename.
            // Transfer the value into the return vec to_reinstall_bad_checksum
            to_reinstall_bad_checksum.push(lockfile_map.remove(file_path).unwrap())
        }
    }
    // lockfile_map is now left with mods which do not exist in the mods folder.
    // Check if the mod is a dependency or not. If so, add to missing_dependencies vec.
    // Otherwise, we check the dependencies of the mods. Check if each dependency is being used by other mods,
    // then safely remove these dependencies along with the missing mod entry.
    for (_, mod_entry) in lockfile_map {
        // Check if the mod is a dependency of any other mods
        let is_dependency = current_lockfile.iter().any(|per_mod| {
            per_mod
                .dependencies
                .iter()
                .any(|dep| dep.project_id == mod_entry.id)
        });
        if is_dependency {
            // Add to missing_dependencies if it's a dependency
            missing_dependencies.push(LockDependency {
                source: mod_entry.source.clone(),
                project_id: mod_entry.id.clone(),
                dependency_type: DependencyType::Required, // Assuming it's a required dependency
            });
        } else {
            // Check dependencies of the mod
            for dependency in &mod_entry.dependencies {
                // Check if the dependency is being used by other mods
                let is_used = current_lockfile.iter().any(|per_mod| {
                    per_mod
                        .dependencies
                        .iter()
                        .any(|dep| dep.project_id == dependency.project_id)
                });

                // If the dependency is not used, remove it from the lockfile
                if !is_used {
                    current_lockfile.retain(|per_mod| per_mod.id != dependency.project_id);
                }
            }
            // Remove the missing mod entry from the lockfile
            current_lockfile.retain(|per_mod| per_mod.id != mod_entry.id);
        }
    }

    // (5) (config to lockfile): Remove mods and its no longer used dependencies if config does not include the mod. Return list of mods to download if config file
    // contains a mod that the lockfile doesn't.
    // Iterate over mods in the lockfile
    for lock_mod in current_lockfile.clone() {
        // Check if the mod is present in the config
        if !config.mods.iter().any(|per_mod| per_mod.id == lock_mod.id) {
            // Mod is not present in the config, remove it from the lockfile
            current_lockfile.retain(|per_mod| per_mod.id != lock_mod.id);

            // Check dependencies of the mod
            for dependency in &lock_mod.dependencies {
                // Check if the dependency is being used by other mods
                let is_used = current_lockfile.iter().any(|per_mod| {
                    per_mod
                        .dependencies
                        .iter()
                        .any(|dep| dep.project_id == dependency.project_id)
                });

                // If the dependency is not used, remove it from the lockfile
                if !is_used {
                    current_lockfile.retain(|per_mod| per_mod.id != dependency.project_id);
                }
            }
        }
    }

    // Check for mods in the config that are not in the lockfile
    for config_mod in &config.mods {
        if !current_lockfile
            .iter()
            .any(|per_mod| per_mod.id == config_mod.id)
        {
            // Mod in the config is not in the lockfile, add it to the list of mods to download
            new_mods.push(config_mod.clone());
        }
    }

    // Finally, save config and lockfile.
    save_config(current_directory, &config)?;
    save_lockfile(current_directory, &current_lockfile)?;

    Ok(SyncFilesReturn {
        missing_dependencies,
        new_mods,
        to_reinstall_bad_checksum,
    })
}
