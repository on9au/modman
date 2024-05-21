use std::{fs, io::{self, BufRead}, path::Path};

use crate::errors::ModManError;

const CONFIG_DIR: &str = r#"C:\ProgramData\winforge\config"#;
const SOURCES_FILE: &str = r#"C:\ProgramData\winforge\config\sources"#;

pub fn write_sources_to_config(sources: &Vec<String>) -> Result<(), ModManError> {
    // Ensure the config directory exists
    if let Err(err) = fs::create_dir_all(CONFIG_DIR) {
        return Err(ModManError::IoError(err));
    }

    // Read existing sources from file
    let existing_sources = read_existing_sources().unwrap_or_default();

    // Merge new sources with existing sources, removing duplicates
    let mut all_sources: Vec<String> = existing_sources.into_iter().chain(sources.iter().cloned()).collect();
    all_sources.sort();
    all_sources.dedup();

    // Write contents to file
    let file_content = all_sources.join("\n");
    if let Err(err) = fs::write(SOURCES_FILE, file_content) {
        return Err(ModManError::IoError(err));
    }

    Ok(())
}

pub fn read_existing_sources() -> io::Result<Vec<String>> {
    let mut sources = Vec::new();
    if Path::new(SOURCES_FILE).exists() {
        let file = fs::File::open(SOURCES_FILE)?;
        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            sources.push(line?);
        }
    }
    Ok(sources)
}