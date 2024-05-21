mod commands;
mod config;
mod errors;

use std::{env, process};
use commands::command_handler;

fn main() {
    let handler = command_handler::handle_command(env::args());

    match handler {
        Ok(()) => process::exit(0),
        Err(e) => process::exit(e.exit_code()),
    }
}