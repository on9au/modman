use reqwest::Client;
use serde::Deserialize;

use crate::datatypes::{Mod, ModSources};

const MODRINTH_API_BASE: &str = "https://api.modrinth.com/v2";

#[derive(Deserialize)]
pub struct ModrinthMod {
    pub id: String,
    pub slug: String,
    pub title: String,
}

pub async fn fetch_modrinth_mod(client: &Client, id_slug: &str) -> Result<Mod, reqwest::Error> {
    let url = format!("{}/v2/project/{}", MODRINTH_API_BASE, id_slug);
    let response = client.get(&url).send().await?.json::<ModrinthMod>().await?;
    let result = Mod {
        source: ModSources::Modrinth,
        id: response.id,
        name: response.title,
    };
    Ok(result)
}