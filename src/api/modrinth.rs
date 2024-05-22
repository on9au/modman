use reqwest::{Client, StatusCode};
use serde::Deserialize;

use crate::datatypes::{Mod, ModSources};

const MODRINTH_API_BASE: &str = "https://api.modrinth.com";

#[derive(Debug, Deserialize)]
pub struct License {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DonationUrl {
    pub id: String,
    pub platform: String,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct GalleryItem {
    pub url: String,
    pub featured: bool,
    pub title: String,
    pub description: String,
    pub created: Option<String>,
    pub ordering: i32,
}

#[derive(Debug, Deserialize)]
pub struct ModrinthMod {
    pub slug: String,
    pub title: String,
    pub id: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
}

pub async fn fetch_modrinth_mod(client: &Client, id_slug: &str) -> Result<Mod, Box<dyn std::error::Error>> {
    let url = format!("{}/v2/project/{}", MODRINTH_API_BASE, id_slug);
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
            let modrinth_mod = response.json::<ModrinthMod>().await?;
            let result = Mod {
                source: ModSources::Modrinth,
                id: modrinth_mod.id,
                name: modrinth_mod.title,
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