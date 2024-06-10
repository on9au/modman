use colored::Colorize;

use crate::errors::ModManError;

pub fn command_version() -> Result<(), ModManError> {
    println!(
        "{} ({})",
        "ModMan".yellow(),
        env!("CARGO_PKG_NAME").bright_black()
    );
    println!("v{}", env!("CARGO_PKG_VERSION"));
    println!("{}", env!("CARGO_PKG_DESCRIPTION"));
    println!("Author: {}", env!("CARGO_PKG_AUTHORS"));

    Ok(())
}
