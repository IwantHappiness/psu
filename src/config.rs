#![allow(unused)]
use anyhow::{Context, Error, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};

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
			path: dirs::home_dir().unwrap_or_default(),
			..Default::default()
		}
	}

	pub fn gen_config() -> Result<()> {
		let dir = get_app_data_dir().context("Failed to obtain config directory.")?;

		if !dir.exists() {
			fs::create_dir_all(&dir).context("Failed create config dirs")?;
		}

		let conf = toml::to_string_pretty(&Config::new()).context("Failed to parse configuration.")?;
		fs::write(dir.join(CONFIG), conf).context("Failed write config.")?;

		Ok(())
	}
}

fn get_app_data_dir() -> Option<PathBuf> {
	Some(dirs::config_dir()?.join(APP_NAME))
}

pub fn read_config() -> Result<Config, Error> {
	let path = get_app_data_dir().unwrap().join(CONFIG);

	if !path.exists() {
		Config::gen_config()?;
	}

	let mut config = toml::from_str::<Config>(&fs::read_to_string(path)?)?;
	config.path = config
		.path
		.as_os_str()
		.to_string_lossy()
		.replace("~/", &dirs::home_dir().unwrap().to_string_lossy())
		.into();

	Ok(config)
}
