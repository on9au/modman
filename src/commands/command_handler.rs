use std::env::Args;

use colored::Colorize;

use crate::{commands::{
    init, command_structs, install, version
}, errors::ModManError};

pub fn handle_command(mut args: Args) -> Result<(), ModManError>{
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
        return Ok(())
    }

    // Parse the remaining arguments
    while let Some(arg) = args.next() {
        if arg.starts_with("--") {
            command_options.flags.push(arg);
        } else {
            command_options.parameters.push(arg);
        }
    }

    let command_result: Result<(), ModManError> = match command_options.command.to_lowercase().as_str() {
        "help" => command_help(),
        "version" => version::command_version(),
        "install" => install::command_install(&command_options),
        "add" => todo!(),
        "init" => init::command_init(),
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
        return Err(e)
    }

    Ok(())
}

fn command_help() -> Result<(), ModManError> {
    println!("HELP...");
    Ok(())
}