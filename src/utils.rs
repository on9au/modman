use std::{env, path::PathBuf};

pub fn get_current_working_dir() -> std::io::Result<PathBuf> {
    env::current_dir()
}