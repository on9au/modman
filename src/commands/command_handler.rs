use std::env::Args;

use colored::Colorize;

use crate::{
    commands::{add, command_structs, init, version},
    errors::ModManError,
};

use super::sync;

pub async fn handle_command(mut args: Args) -> Result<(), ModManError> {
    args.next(); // Skip first args, which is the program binary.

    // Parse the command and its arguments
    let mut command_options = command_structs::CommandOptions {
        command: String::new(),
        flags: Vec::new(),
        parameters: Vec::new(),
    };

    if let Some(cmd) = args.next() {
        command_options.command = cmd;
    } else {
        println!("No command specified.");
        let _ = command_help();
        return Ok(());
    }

    // Parse the remaining arguments
    for arg in args {
        if arg.starts_with("--") {
            command_options.flags.push(arg);
        } else {
            command_options.parameters.push(arg);
        }
    }

    let command_result: Result<(), ModManError> =
        match command_options.command.to_lowercase().as_str() {
            "help" => command_help(),
            "version" => version::command_version(),
            "install" => todo!(),
            "add" => add::command_add(&command_options).await,
            "init" => init::command_init(),
            "sync" => sync::command_sync(&command_options).await,
            "remove" => todo!(),
            "search" => todo!(),
            "update" => todo!(),
            "upgrade" => todo!(),
            "list" => todo!(),
            "info" => todo!(),
            _ => {
                println!("Unknown command '{}'.", command_options.command);
                command_help()?;
                Err(ModManError::CommandNotFound)
            }
        };

    if let Err(e) = command_result {
        eprintln!("{}: {}", "[Error executing command]".red().bold(), e);
        return Err(e);
    }

    Ok(())
}

fn command_help() -> Result<(), ModManError> {
    println!("HELP...");
    Ok(())
}
