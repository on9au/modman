use futures::StreamExt;
use reqwest::Client;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::{error::Error, fs::File, io::Write, path::PathBuf, sync::Arc};

use colored::Colorize;

use crate::{alert, confirm};

const MAX_MOD_NAME_LENGTH: usize = 30;

// Solve issue with returning string errors
#[derive(Debug)]
struct StrError<'a>(&'a str);
// Error doesn't require you to implement any methods, but
// your type must also implement Debug and Display.
impl<'a> Error for StrError<'a> {}

impl<'a> std::fmt::Display for StrError<'a>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Delegate to the Display impl for `&str`:
        self.0.fmt(f)
    }
}

pub async fn download_mod(
    client: &Client,
    url: &str,
    dest: &PathBuf,
    mod_name: &str,
    multi_pb: &MultiProgress,
) -> Result<(), Box<dyn Error + Send>> {
    let response = match client.get(url).send().await {
        Ok(resp) => resp,
        Err(err) => {
            return Err(Box::new(err));
        }
    };

    let total_size = match response.content_length() {
        Some(size) => size,
        None => {
            return Err(Box::new(StrError("No content length found")));
        }
    };

    let pb = multi_pb.add(ProgressBar::new(total_size));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("##-"),
    );

    let display_name = if mod_name.len() > MAX_MOD_NAME_LENGTH {
        format!("{}...", &mod_name[..MAX_MOD_NAME_LENGTH - 3])
    } else {
        mod_name.to_string()
    };

    pb.set_message(format!("Downloading: {}", display_name));

    let mut file = match File::create(dest) {
        Ok(result) => result,
        Err(err) => {
            pb.finish_with_message(format!("Error creating file: {}", err));
            return Err(Box::new(err));
        }
    };

    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(item) = stream.next().await {
        let chunk = match item {
            Ok(result) => result,
            Err(err) => {
                pb.finish_with_message(format!("Error downloading: {}", err));
                return Err(Box::new(err));
            }
        };
        if let Err(e) = file.write_all(&chunk) {
            pb.finish_with_message(format!("Error writing to file: {}", e));
            return Err(Box::new(e));
        }
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }

    pb.finish_with_message(format!("Downloaded: {}", display_name));

    Ok(())
}

// Function to download all mods asynchronously
pub async fn download_all_mods(
    client: &Arc<Client>,
    mods: Vec<(String, PathBuf, String)>,
) -> Result<(), Box<dyn Error + Send>> {
    let multi_pb = MultiProgress::new();
    let mut tasks = Vec::new();

    for (url, dest, name) in mods {
        let client = Arc::clone(client);
        let multi_pb = multi_pb.clone();
        let mod_match = tokio::spawn(async move {
            download_mod(&client, &url, &dest, &name, &multi_pb).await
        });
        tasks.push(mod_match)
    }

    let results = futures::future::join_all(tasks).await;

    for result in results {
        match result {
            Ok(Ok(())) => { /* Success, do nothing */ }
            Ok(Err(e)) => {
                let message = "Error downloading mod: ".to_string() + &e.to_string();
                alert!(message);
            }
            Err(e) => return Err(Box::new(e)),
        }
    }

    Ok(())
}