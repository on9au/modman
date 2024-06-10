use crate::api::modrinth::fetch_modrinth_mod;
use crate::datatypes::{DependencyType, GameLoader, LockDependency, LockMod};
use crate::errors::ModManError;
use reqwest::Client;
use std::pin::Pin;

pub async fn handle_dependencies(
    client: &Client,
    mods_to_install: &mut Vec<LockMod>,
    dependencies: &[LockDependency],
    minecraft_version: &String,
    loader: &GameLoader,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut tasks = Vec::new();

    for dep in dependencies {
        if !mods_to_install.iter().any(|m| m.id == dep.project_id) {
            match dep.dependency_type {
                DependencyType::Required => {
                    let client = client.clone();
                    let dep_id = dep.project_id.clone();
                    let minecraft_version = minecraft_version.clone();
                    let loader = loader.clone();

                    tasks.push(tokio::spawn(async move {
                        fetch_modrinth_mod(&client, &dep_id, &minecraft_version, &loader).await
                    }));
                }
                DependencyType::Optional => {
                    // Optional dependencies can be skipped or handled differently if needed
                }
                DependencyType::Incompatible => {
                    return Err(Box::new(ModManError::IncompatibleDependency(
                        format!("Incompatible mod: {}", dep.project_id).into(),
                    )));
                }
                DependencyType::Embedded => {
                    // Handle embedded dependencies if needed
                }
            }
        }
    }

    let results = futures::future::join_all(tasks).await;
    for result in results {
        match result {
            Ok(Ok(dep_mod)) => {
                if !mods_to_install.iter().any(|m| m.id == dep_mod.id) {
                    mods_to_install.push(dep_mod.clone());
                    let fut = handle_dependencies(
                        client,
                        mods_to_install,
                        &dep_mod.dependencies,
                        minecraft_version,
                        loader,
                    );
                    Pin::from(Box::new(fut)).await?;
                }
            }
            Ok(Err(e)) => {
                return Err(e);
            }
            Err(join_error) => {
                return Err(Box::new(ModManError::APIFetchError(format!(
                    "Task failed: {:?}",
                    join_error
                ))));
            }
        }
    }
    Ok(())
}
