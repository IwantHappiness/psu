use colored::Colorize;
use psu::{create_table, insert, print, remove, Cli, Commands};
use rusqlite::Connection;

const DB: &str = "password.db";

fn main() {
    let cli = Cli::run();

    let conn = Connection::open(DB).unwrap();

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
