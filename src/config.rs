// #![allow(unused)]
// #![warn(clippy::all, clippy::pedantic)]
use anyhow::{Context, Result};
use config::{Config as ConfigBuilder, ConfigError, File, FileFormat};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

const CONFIG_FILE: &str = "config.toml";
const APP_NAME: &str = "psu";

#[derive(Deserialize, Serialize, Debug)]
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
	pub fn new() -> Result<Self, ConfigError> {
		let mut builder = ConfigBuilder::builder();
		builder = builder.add_source(
			File::from(get_app_data_dir().unwrap().join(CONFIG_FILE))
				.format(FileFormat::Toml)
				.required(false),
		);

		builder.build()?.try_deserialize().map(|mut config: Config| {
			config.replace_tilde();
			config
		})
	}

	pub fn gen_config() -> Result<()> {
		let dir = get_app_data_dir().context("Failed to obtain config directory.")?;

		if !dir.exists() {
			fs::create_dir_all(&dir).context("Failed create config dirs")?;
		}

		let conf = toml::to_string_pretty(&Config::default()).context("Failed to parse configuration.")?;
		fs::write(dir.join(CONFIG_FILE), conf).context("Failed write config.")?;

		Ok(())
	}

	fn replace_tilde(&mut self) {
		if let Some(home) = dirs::home_dir() {
			self.path = self.path.to_string_lossy().replace('~', &home.to_string_lossy()).into();
		}
	}
}

impl Default for Config {
	fn default() -> Self {
		Self {
			path: dirs::home_dir().unwrap_or_default(),
			fields: Default::default(),
		}
	}
}

fn get_app_data_dir() -> Option<PathBuf> {
	Some(dirs::config_dir()?.join(APP_NAME))
}

#[cfg(test)]
mod test {
	use super::Config;

	#[test]
	fn rep_tilde() {
		if let Some(home) = dirs::home_dir() {
			let mut conf = Config {
				path: "~/Downloads".into(),
				..Default::default()
			};
			conf.replace_tilde();
			// println!("{}, {}", conf.path.display(), home.join("Downloads").display());
			assert_eq!(conf.path, home.join("Downloads"))
		}
	}
}
