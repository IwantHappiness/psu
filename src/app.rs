// #![warn(clippy::all, clippy::pedantic)]
use super::config::Config;
use super::ui::TableColors;
use anyhow::{Context, Result};
use clipboard::{ClipboardContext, ClipboardProvider};
use csv::Writer;
use ratatui::{
	style::palette::tailwind,
	widgets::{ScrollbarState, TableState},
};
use serde::{Deserialize, Serialize};
use std::{
	borrow::Borrow,
	error::Error,
	fmt::Display,
	fs::{self, File},
	path::Path,
};
use tui_input::Input;

pub const PASSWORD_FILE: &str = "psu.csv";
pub const TEMP_FILE: &str = "psu.csv.temp";
pub const ITEM_HEIGHT: usize = 3;

#[derive(Debug, Default, PartialEq)]
pub enum CurrentScreen {
	#[default]
	Main,
	Popup,
	Help,
}

#[derive(Debug, Default, PartialEq)]
pub enum InputMode {
	#[default]
	Service,
	Login,
	Password,
}

#[derive(Default)]
pub struct App {
	pub config: Config,
	// User form
	pub input: UserInput,
	// Vector with passwords
	pub items: Vec<Password>,
	// Current field
	pub input_mode: InputMode,
	// Table
	pub state: TableState,
	// Pallette for table
	pub colors: TableColors,
	// Scroll State
	pub scroll_state: ScrollbarState,
	// Current input mode
	pub current_screen: CurrentScreen,
	// Need for handle mode input password
	pub is_modify: bool,
}

impl App {
	pub fn new() -> Self {
		let config = Config::new().unwrap_or_else(|_| {
			Config::gen_config().unwrap();
			Config::new().unwrap_or_default()
		});
		let items = App::read(config.path.join(PASSWORD_FILE)).unwrap_or_default();

		Self {
			config,
			input: UserInput::default(),
			input_mode: InputMode::default(),
			current_screen: CurrentScreen::default(),
			state: TableState::default().with_selected(0),
			colors: TableColors::new(&tailwind::GRAY),
			scroll_state: ScrollbarState::new(items.len().saturating_sub(ITEM_HEIGHT)),
			is_modify: false,
			items,
		}
	}

	pub fn next_row(&mut self) {
		let i = match self.state.selected() {
			Some(i) => {
				if i >= self.items.len() - 1 {
					0
				} else {
					i + 1
				}
			}
			None => 0,
		};
		self.state.select(Some(i));
		self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
	}

	pub fn previous_row(&mut self) {
		let i = match self.state.selected() {
			Some(i) => {
				if i == 0 {
					self.items.len() - 1
				} else {
					i - 1
				}
			}
			None => 0,
		};
		self.state.select(Some(i));
		self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
	}

	pub fn next_input_mode(&mut self) {
		self.input_mode = match self.input_mode {
			InputMode::Login => InputMode::Password,
			InputMode::Password => InputMode::Service,
			InputMode::Service => InputMode::Login,
		};
	}

	pub fn prev_input_mode(&mut self) {
		self.input_mode = match self.input_mode {
			InputMode::Login => InputMode::Service,
			InputMode::Password => InputMode::Login,
			InputMode::Service => InputMode::Password,
		};
	}

	pub fn nex_column(&mut self) {
		self.state.select_next_column();
	}

	pub fn previous_column(&mut self) {
		self.state.select_previous_column();
	}

	pub fn add_password(&mut self) {
		let (service, login, password) = self.input.ref_array().into();

		if self.is_modify {
			if let Some(index) = self.state.selected() {
				let data = &mut self.items[index];

				data.login = login.into();
				data.password = password.into();
				data.service = service.into();
			}
		} else {
			let new_id = self.items.len() as u32;
			let data = Password::new(new_id, service, login, password);
			self.items.push(data);
		}
	}

	// Write password to PASSWORD_FILE
	pub fn write(&mut self) -> Result<()> {
		let final_path = self.config.path.join(PASSWORD_FILE);
		let temp_path = self.config.path.join(TEMP_FILE);

		create_csv_file(&temp_path)?;
		let mut wtr = get_writer(&temp_path)?;

		for (index, password) in self.items.iter_mut().enumerate() {
			if password.id != index as u32 {
				password.id = index as u32;
			}

			wtr.write_record(password.ref_array())?;
		}
		wtr.flush()?;

		fs::rename(temp_path, final_path)?;
		Ok(())
	}

	pub fn read<T: AsRef<Path>>(path: T) -> Option<Vec<Password>> {
		if let Ok(mut wtr) = csv::Reader::from_path(path) {
			let vec: Vec<Password> = wtr.deserialize::<Password>().flatten().collect();
			return Some(vec);
		}

		None
	}

	pub fn delete(&mut self) {
		if let Some(index) = self.state.selected() {
			self.items.remove(index);
		}
	}

	#[inline]
	pub fn modify(&mut self) {
		if let Some(index) = self.state.selected() {
			let data = &self.items[index];
			self.input = data.into();
		}
		self.is_modify = true;
	}

	pub fn clip_row(&self) -> anyhow::Result<(), Box<dyn Error>> {
		if let Some(index) = self.state.selected() {
			let mut ctx = ClipboardContext::new()?;
			let password = self.items.get(index).context("No get Password.")?;

			ctx.set_contents(password.to_string())?;
		}

		Ok(())
	}

	pub fn clip_password(&self) -> anyhow::Result<(), Box<dyn Error>> {
		if let Some(index) = self.state.selected() {
			let mut ctx = ClipboardContext::new()?;
			let password = self.items.get(index).context("No get Password.")?;

			ctx.set_contents(password.password().into())?;
		}

		Ok(())
	}

	pub fn clip_column(&self) -> anyhow::Result<(), Box<dyn Error>> {
		if let Some(index) = self.state.selected() {
			let mut ctx = ClipboardContext::new()?;
			let password = self.items.get(index).context("No get Password.")?;

			let data = match self.state.selected_column() {
				Some(1) => password.login(),
				Some(0) => password.service(),
				_ => password.password(),
			};

			ctx.set_contents(data.into())?;
		}

		Ok(())
	}
}

fn get_writer<T: AsRef<Path>>(path: T) -> Result<Writer<File>> {
	Ok(csv::WriterBuilder::new()
		.delimiter(b',')
		.quote_style(csv::QuoteStyle::NonNumeric)
		.from_writer(std::fs::OpenOptions::new().create(true).append(true).open(path)?))
}

fn create_csv_file<T: AsRef<Path>>(path: T) -> Result<()> {
	let mut wtr = get_writer(path)?;
	wtr.write_record(["Id", "Service", "Login", "Password"])?;
	wtr.flush()?;

	Ok(())
}

pub trait Data {
	fn login(&self) -> &str;

	fn password(&self) -> &str;

	fn service(&self) -> &str;
}

#[derive(Clone, Debug, Default)]
pub struct UserInput {
	pub service: Input,
	pub login: Input,
	pub password: Input,
}

impl UserInput {
	pub fn reset_data(&mut self) {
		self.login.reset();
		self.password.reset();
		self.service.reset();
	}

	pub fn ref_array(&self) -> [&str; 3] {
		[self.service(), self.login(), self.password()]
	}

	pub fn from_array<T: AsRef<str>>(value: &[T; 3]) -> Self {
		Self {
			service: value[0].as_ref().into(),
			login: value[1].as_ref().into(),
			password: value[2].as_ref().into(),
		}
	}
}

impl Data for UserInput {
	fn login(&self) -> &str {
		self.login.value()
	}

	fn password(&self) -> &str {
		self.password.value()
	}

	fn service(&self) -> &str {
		self.service.value()
	}
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Password {
	pub id: u32,
	pub service: String,
	pub login: String,
	pub password: String,
}

impl Password {
	pub fn new<T: AsRef<str>>(id: u32, service: T, login: T, password: T) -> Self {
		Self {
			id,
			service: service.as_ref().into(),
			login: login.as_ref().into(),
			password: password.as_ref().into(),
		}
	}

	pub fn id(&self) -> String {
		self.id.to_string()
	}

	pub fn ref_array(&self) -> [String; 4] {
		[
			self.id(),
			self.service().into(),
			self.login().into(),
			self.password().into(),
		]
	}
}

impl Display for Password {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}   {}   {}", self.service, self.login, self.password,)
	}
}

impl Data for Password {
	fn login(&self) -> &str {
		&self.login
	}
	fn password(&self) -> &str {
		&self.password
	}

	fn service(&self) -> &str {
		&self.service
	}
}

impl<T: Borrow<Password>> From<T> for UserInput {
	fn from(value: T) -> Self {
		let value = value.borrow();
		Self {
			service: value.service().into(),
			login: value.login().into(),
			password: value.password().into(),
		}
	}
}

#[cfg(test)]
mod test {
	use crate::app::Password;

	use super::UserInput;

	#[test]
	fn user_input_ref_array() {
		let user_input = UserInput::from_array(&["a", "b", "c"]);
		assert_eq!(user_input.ref_array(), ["a", "b", "c"])
	}

	#[test]
	fn password_ref_array() {
		let user_input = Password::new(0, "a", "b", "c");
		assert_eq!(user_input.ref_array(), ["0", "a", "b", "c"])
	}
}
