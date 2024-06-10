use std::{env, path::PathBuf};

use crate::datatypes::{Config, LockMod};

pub fn get_current_working_dir() -> std::io::Result<PathBuf> {
    env::current_dir()
}

pub fn convert_lock_mods_to_tuples(
    config: &Config,
    lock_mods: Vec<LockMod>,
) -> Vec<(String, PathBuf, String, String)> {
    lock_mods
        .into_iter()
        .map(|lock_mod| {
            let url = lock_mod.download_url;
            let name = lock_mod.name;
            let dest = config.mods_folder.join(&lock_mod.file_name);
            let hash = lock_mod.sha512;
            (url, dest, name, hash)
        })
        .collect()
}

pub fn calculate_total_size(mods_to_install: &[LockMod]) -> String {
    let total_size: u64 = mods_to_install.iter().map(|mod_| mod_.size).sum();
    let mut total_size_str = String::new();

    if total_size < 1024 {
        total_size_str.push_str(&format!("{} B", total_size));
    } else if total_size < 1024 * 1024 {
        total_size_str.push_str(&format!("{:.2} KB", total_size as f64 / 1024.0));
    } else if total_size < 1024 * 1024 * 1024 {
        total_size_str.push_str(&format!("{:.2} MB", total_size as f64 / (1024.0 * 1024.0)));
    } else {
        total_size_str.push_str(&format!(
            "{:.2} GB",
            total_size as f64 / (1024.0 * 1024.0 * 1024.0)
        ));
    }

    total_size_str
}
