use anyhow::{Context, Result};
use clipboard::{ClipboardContext, ClipboardProvider};
use csv::Writer;
use ratatui::{
	style::{Color, palette::tailwind},
	widgets::{ScrollbarState, TableState},
};
use serde::{Deserialize, Serialize};
use std::{
	error::Error,
	fs::{self, File},
	path::Path,
};
use tui_input::Input;

const PASSWORD_FILE: &str = "psu.csv";
const TEMP_FILE: &str = "psu.csv.temp";
pub const ITEM_HEIGHT: usize = 3;

#[derive(Debug, Default, PartialEq)]
pub enum CurrentScreen {
	#[default]
	Main,
	Popup,
}

#[derive(Debug, Default, PartialEq)]
pub enum InputMode {
	#[default]
	Login,
	Password,
	Service,
}

#[derive(Default)]
pub struct App {
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
		let data = App::read(PASSWORD_FILE).unwrap_or_default();

		Self {
			input: UserInput::default(),
			input_mode: InputMode::default(),
			current_screen: CurrentScreen::default(),
			state: TableState::default().with_selected(0),
			colors: TableColors::new(&tailwind::EMERALD),
			scroll_state: ScrollbarState::new(data.len().saturating_sub(ITEM_HEIGHT)),
			is_modify: false,
			items: data,
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
		let (login, password, service) = self.input.ref_array().into();

		if self.is_modify {
			if let Some(index) = self.state.selected() {
				let data = &mut self.items[index];

				data.login = login.into();
				data.password = password.into();
				data.service = service.into();
			}
		} else {
			let new_id = self.items.len() as u32;
			let data = Password::new(new_id, login, password, service);
			self.items.push(data);
		}
	}

	// Write password to PASSWORD_FILE
	pub fn write(&mut self) -> Result<()> {
		create_csv_file(TEMP_FILE)?;
		let mut wtr = get_writer(TEMP_FILE)?;

		for (index, password) in self.items.iter_mut().enumerate() {
			if password.id != index as u32 {
				password.id = index as u32;
			}

			wtr.write_record(password.ref_array())?;
		}
		wtr.flush()?;

		fs::rename(TEMP_FILE, PASSWORD_FILE)?;
		Ok(())
	}

	pub fn read<T: AsRef<Path>>(path: T) -> Option<Vec<Password>> {
		if let Ok(mut wtr) = csv::Reader::from_path(path) {
			let vec: Vec<Password> = wtr.deserialize::<Password>().flatten().collect();
			return Some(vec);
		};

		None
	}

	pub fn delete(&mut self) {
		if let Some(index) = self.state.selected() {
			self.items.remove(index);
		};
	}

	pub fn modify(&mut self) {
		if let Some(index) = self.state.selected() {
			let data = &self.items[index];
			self.input = data.into();
		}
		self.is_modify = true;
	}

	pub fn clip(&self) -> anyhow::Result<(), Box<dyn Error>> {
		if let Some(index) = self.state.selected() {
			let mut ctx = ClipboardContext::new()?;
			let password = self.items.get(index).context("No get Password.")?;

			let data = match self.state.selected_column() {
				Some(0) => password.login(),
				Some(1) => password.password(),
				Some(2) => password.service(),
				_ => password.password(),
			};

			ctx.set_contents(data.into())?;
		}

		Ok(())
	}

	pub fn clip_all(&self) -> anyhow::Result<(), Box<dyn Error>> {
		if let Some(index) = self.state.selected() {
			let mut ctx = ClipboardContext::new()?;
			let password = self.items.get(index).context("No get Password.")?;

			ctx.set_contents(format!(
				"{}, {}, {}",
				password.login, password.password, password.service
			))?;
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
	wtr.write_record(["Id", "Login", "Password", "Service"])?;
	wtr.flush()?;

	Ok(())
}

#[derive(Default)]
pub struct TableColors {
	pub alt_row_color: Color,
	pub buffer_bg: Color,
	pub footer_border_color: Color,
	pub header_bg: Color,
	pub header_fg: Color,
	pub normal_row_color: Color,
	pub row_fg: Color,
	pub selected_cell_style_fg: Color,
	pub selected_column_style_fg: Color,
	pub selected_row_style_fg: Color,
}

impl TableColors {
	const fn new(color: &tailwind::Palette) -> Self {
		Self {
			buffer_bg: tailwind::SLATE.c950,
			header_bg: color.c900,
			header_fg: tailwind::SLATE.c200,
			row_fg: tailwind::SLATE.c200,
			selected_row_style_fg: color.c400,
			selected_column_style_fg: color.c400,
			selected_cell_style_fg: color.c600,
			normal_row_color: tailwind::SLATE.c950,
			alt_row_color: tailwind::SLATE.c900,
			footer_border_color: color.c400,
		}
	}
}

pub trait Data {
	fn login(&self) -> &str;

	fn password(&self) -> &str;

	fn service(&self) -> &str;
}

#[derive(Clone, Debug, Default)]
pub struct UserInput {
	pub login: Input,
	pub password: Input,
	pub service: Input,
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

impl UserInput {
	pub fn reset_data(&mut self) {
		self.login.reset();
		self.password.reset();
		self.service.reset();
	}

	fn ref_array(&self) -> [&str; 3] {
		[self.login.value(), self.password.value(), self.service.value()]
	}
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Password {
	pub id: u32,
	pub login: String,
	pub password: String,
	pub service: String,
}

#[allow(unused)]
impl Password {
	pub fn new<T: AsRef<str>>(id: u32, login: T, password: T, service: T) -> Self {
		Self {
			id,
			login: login.as_ref().into(),
			password: password.as_ref().into(),
			service: service.as_ref().into(),
		}
	}

	pub fn id(&self) -> String {
		self.id.to_string()
	}

	pub fn ref_array(&self) -> [String; 4] {
		[
			self.id(),
			self.login().into(),
			self.password().into(),
			self.service().into(),
		]
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

impl From<Password> for UserInput {
	fn from(password: Password) -> Self {
		UserInput {
			login: password.login().into(),
			password: password.password().into(),
			service: password.service().into(),
		}
	}
}

impl From<&Password> for UserInput {
	fn from(password: &Password) -> Self {
		UserInput {
			login: password.login().into(),
			password: password.password().into(),
			service: password.service().into(),
		}
	}
}

impl From<[String; 3]> for UserInput {
	fn from(value: [String; 3]) -> Self {
		UserInput {
			login: value[0].as_str().into(),
			password: value[1].as_str().into(),
			service: value[2].as_str().into(),
		}
	}
}
