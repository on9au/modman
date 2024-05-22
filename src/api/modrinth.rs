use reqwest::{Client, StatusCode};
use serde::Deserialize;

use crate::datatypes::{Mod, ModSources, GameLoader};

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
    // date_published: String,
    // downloads: u64,
    // changelog_url: Option<String>,
    // files: Vec<File>,
}

#[derive(Debug, Deserialize)]
struct Dependency {
    version_id: String,
    project_id: String,
    file_name: String,
    dependency_type: String,
}

#[derive(Debug, Deserialize)]
struct File {
    hashes: Hashes,
    url: String,
    filename: String,
    primary: bool,
    size: u64,
    file_type: String,
}

#[derive(Debug, Deserialize)]
struct Hashes {
    sha512: String,
    sha1: String,
}

pub async fn fetch_modrinth_mod(client: &Client, id_slug: &str, minecraft_version: &String, loader: &GameLoader) -> Result<Mod, Box<dyn std::error::Error>> {
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
            let result = Mod {
                source: ModSources::Modrinth,
                id: modrinth_mod.first().unwrap().project_id.clone(),
                name: modrinth_mod.first().unwrap().name.clone(),
            };
            Ok(result)
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