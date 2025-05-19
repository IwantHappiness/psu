use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::{
	fs::{self},
	io,
};
use tui_input::Input;

const PASSWORD_FILE: &str = "psu.csv";

#[derive(Default)]
pub struct App {
	// Data form
	pub data: Data,
	// Current field
	pub data_mode: DataMode,
	// Current input mode
	pub current_screen: CurrentScreen,
}

#[derive(Debug, Default, PartialEq)]
pub enum CurrentScreen {
	#[default]
	Main,
	Editing,
	Exiting,
}

#[derive(Debug, Default, PartialEq)]
pub enum DataMode {
	#[default]
	Login,
	Password,
	Service,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Data {
	pub login: Input,
	pub password: Input,
	pub service: Input,
}

impl Data {
	#[allow(unused)]
	fn new() -> Self {
		Data {
			login: Input::default(),
			password: Input::default(),
			service: Input::default(),
		}
	}

	pub fn reset_data(&mut self) {
		self.login.reset();
		self.password.reset();
		self.service.reset();
	}

	fn ref_array(&self) -> [&str; 3] {
		[self.login.value(), self.password.value(), self.service.value()]
	}
}

impl App {
	pub fn new() -> Self {
		Self {
			data: Data::default(),
			data_mode: DataMode::default(),
			current_screen: CurrentScreen::default(),
		}
	}

	// Print password to PASSWORD_FILE
	pub fn print(&mut self) -> Result<(), io::Error> {
		let mut wtr = csv::WriterBuilder::new()
			.delimiter(b',')
			.quote_style(csv::QuoteStyle::NonNumeric)
			.from_writer(
				std::fs::OpenOptions::new()
					.append(true)
					.create(true)
					.open(PASSWORD_FILE)?,
			);

		// Check file is empty
		if fs::metadata(PASSWORD_FILE).map(|f| f.len() == 0).unwrap_or(true) {
			wtr.write_record(["Login", "Password", "Service"])?;
		}

		wtr.write_record(self.data.ref_array())?;
		wtr.flush()?;

		Ok(())
	}
}
