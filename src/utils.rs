use std::{env, path::PathBuf};

use crate::datatypes::{Config, LockMod};

pub fn get_current_working_dir() -> std::io::Result<PathBuf> {
    env::current_dir()
}

pub fn convert_lock_mods_to_tuples(config: &Config, lock_mods: Vec<LockMod>) -> Vec<(String, PathBuf, String)> {
    lock_mods.into_iter().map(|lock_mod| {
        let url = lock_mod.download_url;
        let name = lock_mod.name;
        let dest = config.mods_folder.join(&lock_mod.file_name);
        (url, dest, name)
    }).collect()
}