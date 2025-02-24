use clap::Parser;
use colored::Colorize;
use rusqlite::{Connection, Error, params};
use std::process;

#[derive(Debug, PartialEq, Clone, Parser)]
pub struct Password {
    pub id: u16,
    pub service: String,
    pub login: String,
    pub password: String,
}

impl Password {
    pub fn modify(&self, conn: &Connection) {
        let old_password = search_by_id(conn, self.id).unwrap();

        conn.execute(
            "UPDATE password SET service = ?1, login = ?2, password = ?3 WHERE id = ?4",
            params![self.service, self.login, self.password, self.id],
        )
        .unwrap();
        println!(
            "Old password: {} {} {} {}",
            old_password.id.to_string().red(),
            old_password.service.to_string().red(),
            old_password.login.to_string().red(),
            old_password.password.to_string().red()
        );

        println!(
            "New password: {} {} {} {}",
            self.id.to_string().green(),
            self.service.to_string().green(),
            self.login.to_string().green(),
            self.password.to_string().green()
        );
    }
}

pub fn create_table(file: &Connection) -> Result<(), Error> {
    file.execute(
        "CREATE TABLE IF NOT EXISTS password (
            id INTEGER PRIMARY KEY,
            service TEXT NOT NULL,
            login TEXT NOT NULL,
            password TEXT NOT NULL
        )",
        (), // Создание пустой таблицы
    )?;
    Ok(())
}

pub fn insert(file: &Connection, service: &str, login: &str, password: &str) -> Result<(), Error> {
    file.execute(
        "INSERT INTO password (service, login, password) VALUES (?1, ?2, ?3)",
        params![service, login, password],
    )?;

    println!(
        "Add: {} {} {} .",
        service.green(),
        login.green(),
        password.green()
    );

    Ok(())
}

pub fn search_by_id(conn: &Connection, id: u16) -> Option<Password> {
    let query = "SELECT id, service, login, password FROM password WHERE id = ?1".to_string();

    let mut stmt = conn.prepare(&query).unwrap();
    let mut users: Vec<_> = stmt
        .query_map([id], |row| {
            Ok(Password {
                id: row.get(0).unwrap(),
                service: row.get(1).unwrap(),
                login: row.get(2).unwrap(),
                password: row.get(3).unwrap(),
            })
        })
        .unwrap()
        .filter_map(|f| f.ok())
        .collect();

    if !users.is_empty() {
        Some(users.remove(0))
    } else {
        // eprintln!("Password not found");
        // process::exit(1)
        None
    }
}

pub fn search(conn: &Connection) -> Option<Vec<Password>> {
    let query = "SELECT * FROM password".to_string();

    let mut stmt = conn.prepare(&query).unwrap();
    let users: Vec<Password> = stmt
        .query_map([], |row| {
            Ok(Password {
                id: row.get(0).unwrap(),
                service: row.get(1).unwrap(),
                login: row.get(2).unwrap(),
                password: row.get(3).unwrap(),
            })
        })
        .unwrap()
        .filter_map(|f| f.ok())
        .collect();

    match !users.is_empty() {
        true => Some(users),
        false => None,
    }
}

pub fn print(conn: &Connection, all: bool, id: Option<u16>) {
    match !all {
        true => {
            if let Some(id) = id {
                let user = match search_by_id(conn, id) {
                    Some(p) => p,
                    None => {
                        eprintln!("{}", "The password could not be found".red());
                        process::exit(1);
                    }
                };
                // Выводим результаты
                println!(
                    "ID: {}, Service: {}, Login: {}, Password: {}",
                    user.id, user.service, user.login, user.password
                );
            }
        }
        false => {
            let users = match search(conn) {
                Some(p) => p,
                None => {
                    eprintln!("{}", "There are no passwords for output".red());
                    process::exit(1);
                }
            };

            // Выводим результаты
            users.iter().for_each(|e| {
                println!(
                    "ID: {}, Service: {}, Login: {}, Password: {}",
                    e.id, e.service, e.login, e.password
                )
            });
        }
    }
}

pub fn remove(conn: &Connection, id: Option<u16>, all: bool) {
    match !all {
        true => {
            if let Some(id) = id {
                if let Some(mut passwords) = search(conn) {
                    passwords.retain(|e| {
                        if e.id == id {
                            println!(
                                "Remove: {} {} {} {}",
                                e.id.to_string().red(),
                                e.service.to_string().red(),
                                e.login.to_string().red(),
                                e.password.to_string().red()
                            );
                        }

                        e.id != id
                    });

                    conn.execute("DELETE FROM password", []).unwrap();

                    for i in passwords {
                        match insert(conn, &i.service, &i.login, &i.password) {
                            Ok(_) => (),
                            Err(e) => {
                                eprintln!("{}", e.to_string().red());
                                process::exit(1);
                            }
                        }
                    }
                } else {
                    println!("{}", "There are no passwords to delete".red());
                }
            }
        }

        false => {
            conn.execute("DELETE FROM password", []).unwrap();
            match create_table(conn) {
                Ok(()) => println!("{}", "Passwords have been deleted".green()),
                Err(e) => {
                    eprintln!("{}", e.to_string().red());
                    process::exit(1);
                }
            }
        }
    }
}

#[test]
fn is_insert() {
    let conn = Connection::open("tests.db").unwrap();
    let p = Password {
        id: 1,
        service: "gog".to_string(),
        login: "man".to_string(),
        password: "1322".to_string(),
    };

    create_table(&conn).unwrap();
    insert(&conn, &p.service, &p.login, &p.password).unwrap();

    let result = search_by_id(&conn, 1).unwrap();

    conn.execute("DELETE FROM password", []).unwrap();

    assert_eq!(result, p)
}

#[test]
fn is_modify() {
    let conn = Connection::open("tests.db").unwrap();
    let r = Password {
        id: 1,
        service: "gog".to_string(),
        login: "man".to_string(),
        password: "1322".to_string(),
    };
    let p = Password {
        id: 1,
        password: "1".to_string(),
        ..r.clone()
    };

    create_table(&conn).unwrap();
    insert(&conn, &r.service, &r.login, &r.password).unwrap();
    p.modify(&conn);

    let result = search_by_id(&conn, 1).unwrap();

    conn.execute("DELETE FROM password", []).unwrap();

    assert_eq!(result, p)
}

#[test]
fn is_remove() {
    let conn = Connection::open("tests.db").unwrap();
    let p = Password {
        id: 1,
        service: "gog".to_string(),
        login: "man".to_string(),
        password: "1322".to_string(),
    };

    create_table(&conn).unwrap();
    insert(&conn, &p.service, &p.login, &p.password).unwrap();
    remove(&conn, Some(1), false);

    let result = search_by_id(&conn, 1);

    conn.execute("DELETE FROM password", []).unwrap();

    assert_eq!(result, None)
}

#[test]
fn is_remove_all() {
    let conn = Connection::open("tests.db").unwrap();
    let p = Password {
        id: 1,
        service: "gog".to_string(),
        login: "man".to_string(),
        password: "1322".to_string(),
    };

    create_table(&conn).unwrap();
    insert(&conn, &p.service, &p.login, &p.password).unwrap();
    remove(&conn, None, true);

    let result = search_by_id(&conn, 1);

    assert_eq!(result, None)
}
