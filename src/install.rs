use futures::StreamExt;
use reqwest::Client;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use sha2::{Digest, Sha512};
use std::{error::Error, fs::{self, File}, io::{self, Read, Write}, path::PathBuf, sync::Arc};
use terminal_size::{Width, terminal_size};

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
    hash: &String,
) -> Result<(), Box<dyn Error + Send>> {
    let response = match client.get(url).send().await {
        Ok(resp) => resp,
        Err(err) => {
            return Err(Box::new(err));
        }
    };

    let terminal_width = match terminal_size() {
        Some((Width(width), _)) => width as usize,
        None => 80, // Default to 80 if terminal size cannot be determined
    };

    let pb = multi_pb.add(ProgressBar::new(terminal_width as u64 - 22)); // Adjust the width here
    pb.set_style(
        ProgressStyle::with_template(" {spinner:.green} [{msg}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    let display_name = if mod_name.len() > 30 {
        format!("{}...", &mod_name[..27])
    } else {
        format!("{:<30}", mod_name)
    };

    pb.set_message(format!("{}", display_name));

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
    match verify_file(&dest, hash) {
        Ok(true) => return Ok(()),
        Ok(false) => {
            pb.finish_with_message(format!("Hash Sum mismatch!"));
            match fs::remove_file(dest) {
                Ok(result) => result,
                Err(err) => {
                    pb.finish_with_message(format!("Error deleting erroneous file: {}", err));
                    return Err(Box::new(err));
                }
            };
            return Err(Box::new(StrError("Hash Sum mismatch!")));
        },
        Err(e) => {
            pb.finish_with_message(format!("Error verifying file: {}", e));
            return Err(Box::new(e));
        },
    };
}

pub async fn download_all_mods(
    client: &Arc<Client>,
    mods: Vec<(String, PathBuf, String, String)>,
) -> Result<(), Box<dyn Error + Send>> {
    let multi_pb = Arc::new(MultiProgress::new());
    let mut tasks = Vec::new();

    for (url, dest, name, hash) in mods {
        let client = Arc::clone(client);
        let multi_pb = Arc::clone(&multi_pb);
        let mod_match = tokio::spawn(async move {
            download_mod(&client, &url, &dest, &name, &multi_pb, &hash).await
        });
        tasks.push(mod_match)
    }

    let _ = futures::future::join_all(tasks).await;
    Ok(())
}

fn calculate_sha512(file_path: &PathBuf) -> Result<String, io::Error> {
    // Open the file
    let mut file = File::open(file_path)?;
    
    // Create a SHA-512 hasher instance
    let mut hasher = Sha512::new();
    
    // Read file in chunks
    let mut buffer = [0; 1024];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    
    // Retrieve the hash result and convert it to a hex string
    let hash_result = hasher.finalize();
    Ok(hex::encode(hash_result))
}

fn verify_file(file_path: &PathBuf, expected_hash: &str) -> Result<bool, io::Error> {
    // Calculate the SHA-512 hash of the file
    let file_hash = calculate_sha512(file_path)?;
    
    // Compare the calculated hash with the expected hash
    Ok(file_hash == expected_hash)
}