mod commands;
mod config;
mod errors;
mod utils;
mod datatypes;

use std::{env, process};
use commands::command_handler;

pub const USER_AGENT: &str = "ModMan/0.1.0 (https://github.com/nulluser0/modman)";

fn main() {
    let handler = command_handler::handle_command(env::args());

    match handler {
        Ok(()) => process::exit(0),
        Err(e) => process::exit(e.exit_code()),
    }
}