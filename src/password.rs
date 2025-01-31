use super::search_by_id;
use clap::Parser;
use colored::Colorize;
use rusqlite::{params, Connection};

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
			"New password : {} {} {} {}",
			self.id.to_string().green(),
			self.service.to_string().green(),
			self.login.to_string().green(),
			self.password.to_string().green()
		);
	}
}
