use cli::{Cli, Commands};
use psu::{create_table, insert, print, remove};
use rusqlite::Connection;

mod cli;

const DB: &str = "password.db";

#[cfg(target_os = "linux")]
pub fn only_linux() -> Connection {
    use std::{fs, path::Path};

    let home_dir = match std::env::var("HOME") {
        Ok(path) => path,
        Err(e) => panic!("HOME : {e}"),
    };

    let path = format!("{home_dir}/Documents");

    if !Path::new(&path).try_exists().unwrap() {
        fs::create_dir(&path).expect("Failed create dir Documents");
    }

    Connection::open(format!("{}/{DB}", path)).expect("Failed to open the database")
}

#[cfg(target_os = "windows")]
pub fn only_windows() -> Connection {
    use directories::UserDirs;
    use std::path::PathBuf;

    if let Some(doc) = UserDirs::new() {
        let doc_path = doc.document_dir().unwrap();
        let path = PathBuf::from(doc_path).join(DB);

        return Connection::open(path).unwrap();
    }

    panic!("Failed to open the database");
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
        } => insert(&conn, &service, &login, &password).unwrap(),

        Commands::Print { all, id } => print(&conn, all, id),

        Commands::Modify(p) => p.modify(&conn),

        Commands::Remove { id, all } => remove(&conn, id, all),
    };
}
