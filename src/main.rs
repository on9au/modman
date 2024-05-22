mod commands;
mod api;
mod config;
mod errors;
mod utils;
mod datatypes;
mod macros;

use std::{env, process};
use commands::command_handler;

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
   let handler = command_handler::handle_command(env::args()).await;

    match handler {
        Ok(()) => process::exit(0),
        Err(e) => process::exit(e.exit_code()),
    }
}