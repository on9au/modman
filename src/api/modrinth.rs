use reqwest::{Client, StatusCode};
use serde::Deserialize;

use crate::datatypes::{GameLoader, LockDependency, LockMod, ModSources};

const MODRINTH_API_BASE: &str = "https://api.modrinth.com";
#[derive(Debug, Deserialize)]
struct ModrinthVersion {
    name: String,
    dependencies: Vec<ModrinthDependency>,
    project_id: String,
    date_published: String,
    files: Vec<File>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModrinthDependency {
    pub project_id: String,
    pub dependency_type: String,
}

#[derive(Debug, Deserialize)]
struct File {
    hashes: Hashes,
    url: String,
    filename: String,
    // primary: bool,
    size: u64,
}

#[derive(Debug, Deserialize)]
struct Hashes {
    sha512: String,
}

pub async fn fetch_modrinth_mod(client: &Client, id_slug: &str, minecraft_version: &String, loader: &GameLoader) -> Result<LockMod, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("{}/v2/project/{}/version?game_versions=[\"{}\"]&loaders=[\"{}\"]", MODRINTH_API_BASE, id_slug, minecraft_version, loader);
    let response = match client.get(&url).send().await {
        Ok(resp) => resp,
        Err(err) => {
            // Handle connection errors, timeouts, etc.
            return Err(Box::new(err));
        }
    };

    match response.status() {
        StatusCode::OK => {
            // The request was successful, deserialize the JSON
            let modrinth_mod = response.json::<Vec<ModrinthVersion>>().await?;
            if let Some(first_mod) = modrinth_mod.first() {
                convert_modrinth_to_lockmod(first_mod)
            } else {
                // Handle empty array case
                let error_msg = format!("( No Mod File ) {}", id_slug);
                Err(error_msg.into())
            }
        }
        StatusCode::NOT_FOUND => {
            // The resource was not found (404)
            let error_msg = format!("(404 Not Found) {}", id_slug);
            Err(error_msg.into())
        }
        _ => {
            // Other non-404 errors
            let error_msg = format!("Received unexpected status code: {}", response.status());
            Err(error_msg.into())
        }
    }
}

fn convert_modrinth_to_lockmod(modrinth_version: &ModrinthVersion) -> Result<LockMod, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(first_file) = modrinth_version.files.first() {
        let dependencies: Result<Vec<LockDependency>, String> = modrinth_version
            .dependencies
            .iter()
            .map(|dep| dep.clone().try_into())
            .collect();

        let lock_mod = LockMod {
            name: modrinth_version.name.clone(),
            source: ModSources::Modrinth,
            id: modrinth_version.project_id.clone(),
            file_name: first_file.filename.clone(),
            release_date: modrinth_version.date_published.clone(),
            sha512: first_file.hashes.sha512.clone(),
            download_url: first_file.url.clone(),
            dependencies: dependencies?,
            size: first_file.size.clone(),
        };

        Ok(lock_mod)
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No files found for mod",
        )))
    }
}