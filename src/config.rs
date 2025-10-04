use anyhow::{Context, Error, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{env, fs, path::PathBuf};

const CONFIG: &str = "config.toml";
const APP_NAME: &str = "psu";

#[derive(Deserialize, Serialize, Debug, Default)]
pub struct Config {
	pub path: PathBuf,
	pub fields: Fields,
}

#[derive(Deserialize, Default, Serialize, Debug)]
pub struct Fields {
	input: String,
	login: String,
	service: String,
}

impl Config {
	pub fn new() -> Self {
		Self {
			path: env::home_dir().unwrap_or_default(),
			..Default::default()
		}
	}

	pub fn gen_config() -> Result<()> {
		let dir = get_app_data_dir().context("Failed to obtain config directory.")?;

		if !dir.exists() {
			fs::create_dir_all(&dir).context("Failed create config dirs")?;
			let conf = toml::to_string_pretty(&Config::new()).context("Failed to parse configuration.")?;
			fs::write(dir.join(CONFIG), conf).context("Failed write config.")?;
		}

		Ok(())
	}
}

fn get_app_data_dir() -> Option<PathBuf> {
	let project_dirs = ProjectDirs::from("com", "", APP_NAME)?;
	Some(directories::ProjectDirs::config_dir(&project_dirs).to_path_buf())
}
