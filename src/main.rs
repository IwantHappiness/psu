use colored::Colorize;
use psu::{create_table, insert, print, remove, Cli, Commands};
use rusqlite::Connection;

const DB: &str = "password.db";

#[cfg(target_os = "linux")]
pub fn only_linux() -> Connection {
    let home_dir = match std::env::var("HOME") {
        Ok(path) => path,
        Err(e) => panic!("Could not find HOME directory: {e}"),
    };

    let path = format!("{home_dir}/Documents/{DB}");
    match Connection::open(path) {
        Ok(e) => e,
        Err(e) => panic!("Error: {e}"),
    }
}

// Todo
#[cfg(target_os = "windows")]
pub fn only_windows() -> Connection {
    Connection::open(DB).unwrap()
}

fn main() {
    let cli = Cli::run();

    #[cfg(target_os = "linux")]
    let conn = only_linux();

    #[cfg(target_os = "windows")]
    let conn = only_windows();

    create_table(&conn).unwrap();

    match cli.command {
        Commands::Add {
            service,
            login,
            password,
        } => {
            insert(&conn, &service, &login, &password).unwrap();
            println!(
                "Add: {} {} {} .",
                service.to_string().green(),
                login.to_string().green(),
                password.to_string().green()
            );
        }

        Commands::Print { all, id } => print(&conn, all, id),

        Commands::Modify(p) => p.modify(&conn),

        Commands::Remove { id, all } => remove(&conn, id, all),
    };
}
