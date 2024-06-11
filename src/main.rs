mod api;
mod commands;
mod config;
mod config_sync;
mod datatypes;
mod errors;
mod install;
mod macros;
mod utils;

use commands::command_handler;
use std::{env, process};

pub static APP_USER_AGENT: &str = concat!(
    env!("CARGO_PKG_AUTHORS"),
    "/",
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    ""
);

#[tokio::main]
async fn main() {
    match color_eyre::install() {
        Ok(_) => {}
        Err(e) => panic!("{}", e),
    };

    let handler = command_handler::handle_command(env::args()).await;

    match handler {
        Ok(()) => process::exit(0),
        Err(e) => process::exit(e.exit_code()),
    }
}
