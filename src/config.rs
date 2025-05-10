use anyhow::{Context, Error, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use toml;

const CONFIG: &str = "config.toml";

#[derive(Deserialize, Serialize, Debug)]
pub struct Config {}

impl Config {
	pub fn new() -> Self {
		Self {}
	}

	pub fn generate_conf(self) -> Result<()> {
		let dir = self.get_app_data_dir().context("Failed to obtain config directory")?;
		let conf = self.parse_config().context("Failed to parse configuration")?;

		fs::create_dir_all(&dir).context("Failed create config dirs")?;
		fs::write(dir.join(CONFIG), conf).context("Failed write config.")?;
		Ok(())
	}

	fn parse_config(self) -> Result<String, Error> {
		Ok(toml::to_string(&self)?)
	}

	fn get_app_data_dir(&self) -> Option<PathBuf> {
		let project_dirs = ProjectDirs::from("com", "", "psu")?;
		Some(project_dirs.data_local_dir().to_path_buf())
	}
}
