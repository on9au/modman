#[macro_export]
macro_rules! actionheader {
    ($msg:expr) => {
        println!(" {}{}{}", "[".yellow().bold(), $msg.yellow().bold(), "]".yellow().bold());
    };
}

#[macro_export]
macro_rules! info {
    ($msg:expr) => {
        println!(" {} {}", "i".cyan().bold(), $msg.bold());
    };
    ($msg:expr, $val:expr) => {
        println!(" {} {} {}", "i".cyan().bold(), $msg.bold(), $val.bright_black());
    };
}

#[macro_export]
macro_rules! alert {
    ($msg:expr) => {
        println!(" {} {}", "!".red().bold(),
            $msg.red().bold(),
        )
    };
}

#[macro_export]
macro_rules! request {
    ($msg:expr, $action:expr) => {
        print!(" {} {} {} {} ", "?".yellow().bold(), $msg.bold(), $action.bright_black(), ">".bright_black());
        io::stdout().flush().unwrap();
    };
}

#[macro_export]
macro_rules! confirm {
    ($msg:expr) => {
        println!("{} {}", "OK".green().bold(), $msg.bold());
    };
}

#[macro_export]
macro_rules! requestconfirm {
    ($msg:expr, $val:expr, $action:expr) => {
        print!("   | {} {} {} {} ", $msg.bold(), $val.yellow().bold(), $action.bright_black(), ">".bright_black());
        io::stdout().flush().unwrap();
    };
}