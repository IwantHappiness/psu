use colored::Colorize;
use psu::{create_table, insert, print, remove, Cli, Commands};
use rusqlite::Connection;

const DB: &str = "password.db";

#[cfg(target_os = "linux")]
pub fn only_linux() -> Connection {
    use std::{fs, path::Path};

    let home_dir = match std::env::var("HOME") {
        Ok(path) => path,
        Err(e) => panic!("Could not find HOME directory: {e}"),
    };

    let path = format!("{home_dir}/Documents");
    let is_dir_exists = Path::new(&path).try_exists().unwrap();

    if is_dir_exists == false {
        fs::create_dir(&path).expect("Failed create dir Documents");
    }

    let path = format!("{home_dir}/Documents/{DB}");
    Connection::open(path).expect("Failed to open the database")
}

#[cfg(target_os = "windows")]
pub fn only_windows() -> Connection {
    use std::path::PathBuf;

    let home_dir = std::env::var("USERPROFILE").expect("Failed to get HOME directory");

    let mut path = PathBuf::from(home_dir);
    path.push("Documents");
    path.push(DB);

    Connection::open(path).expect("Failed to open the database")
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
