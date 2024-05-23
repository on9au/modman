use reqwest::{Client, StatusCode};
use serde::Deserialize;

use crate::datatypes::{GameLoader, LockMod, Mod, ModSources};

const MODRINTH_API_BASE: &str = "https://api.modrinth.com";
#[derive(Debug, Deserialize)]
struct ModrinthVersion {
    name: String,
    // version_number: String,
    // changelog: String,
    // dependencies: Vec<Dependency>,
    // game_versions: Vec<String>,
    // version_type: String,
    // loaders: Vec<String>,
    // featured: bool,
    // status: String,
    // requested_status: String,
    // id: String,
    project_id: String,
    // author_id: String,
    date_published: String,
    // downloads: u64,
    // changelog_url: Option<String>,
    files: Vec<File>,
}

#[derive(Debug, Deserialize)]
struct Dependency {
    version_id: Option<String>,
    project_id: String,
    file_name: Option<String>,
    dependency_type: String,
}

#[derive(Debug, Deserialize)]
struct File {
    hashes: Hashes,
    url: String,
    filename: String,
    // primary: bool,
    size: u64,
    file_type: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Hashes {
    sha512: String,
}

pub async fn fetch_modrinth_mod(client: &Client, id_slug: &str, minecraft_version: &String, loader: &GameLoader) -> Result<LockMod, Box<dyn std::error::Error>> {
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
            // TODO: HANDLE CASES WHERE IT IS EMPTY!!!!!!!!!!!!!!!!!!!!!!!!!!!
            // The request was successful, deserialize the JSON
            let modrinth_mod = response.json::<Vec<ModrinthVersion>>().await?;
            if let Some(first_mod) = modrinth_mod.first() {
                let result = LockMod {
                    source: ModSources::Modrinth,
                    id: first_mod.project_id.clone(),
                    name: first_mod.name.clone(),
                    file_name: first_mod.files.first().unwrap().filename.clone(),
                    release_date: first_mod.date_published.clone(),
                    sha512: first_mod.files.first().unwrap().hashes.sha512.clone(),
                    download_url: first_mod.files.first().unwrap().url.clone(),
                };
                Ok(result)
            } else {
                // Handle empty array case
                let error_msg = format!("{}", id_slug);
                Err(error_msg.into())
            }
        }
        StatusCode::NOT_FOUND => {
            // The resource was not found (404)
            let error_msg = format!("{}", id_slug);
            Err(error_msg.into())
        }
        _ => {
            // Other non-404 errors
            let error_msg = format!("Received unexpected status code: {}", response.status());
            Err(error_msg.into())
        }
    }
}