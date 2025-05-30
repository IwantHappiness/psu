use anyhow::Result;
use csv::Writer;
use ratatui::{
	style::{Color, palette::tailwind},
	widgets::{ScrollbarState, TableState},
};
use serde::{Deserialize, Serialize};
use std::{
	fs::{self, File},
	path::PathBuf,
};
use tui_input::Input;
use unicode_width::UnicodeWidthStr;

const PASSWORD_FILE: &str = "psu.csv";
pub const ITEM_HEIGHT: usize = 3;

#[derive(Debug, Default, PartialEq)]
pub enum CurrentScreen {
	#[default]
	Main,
	Table,
	Editing,
	Exiting,
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

	pub state: TableState,
	pub longest_item_lens: (u16, u16, u16),
	pub items: Vec<Data>,

	pub scroll_state: ScrollbarState,
	// Current field
	pub input_mode: InputMode,
	// Current input mode
	pub current_screen: CurrentScreen,

	pub colors: TableColors,
}

impl App {
	pub fn new() -> Self {
		let data = read().unwrap_or_default();
		Self {
			input: UserInput::default(),
			input_mode: InputMode::default(),
			longest_item_lens: constraint_len_calculator(&data),
			current_screen: CurrentScreen::default(),
			scroll_state: ScrollbarState::new(data.len().saturating_sub(ITEM_HEIGHT)),
			colors: TableColors::new(&tailwind::EMERALD),
			items: data,
			state: TableState::default().with_selected(0),
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

	pub fn nex_column(&mut self) {
		self.state.select_next_column();
	}

	pub fn previous_column(&mut self) {
		self.state.select_previous_column();
	}
}

// Write password to PASSWORD_FILE
pub fn write(app: &App) -> Result<()> {
	let mut wtr = get_writer(PASSWORD_FILE.into())?;

	// SAFETY: because we create the file if it does not exist.
	if fs::metadata(PASSWORD_FILE)?.len() == 0 {
		wtr.write_record(["Login", "Password", "Service"])?;
	}

	wtr.write_record(app.input.ref_array())?;
	wtr.flush()?;

	Ok(())
}

fn get_writer(path: PathBuf) -> Result<Writer<File>> {
	Ok(csv::WriterBuilder::new()
		.delimiter(b',')
		.quote_style(csv::QuoteStyle::NonNumeric)
		.from_writer(std::fs::OpenOptions::new().create(true).append(true).open(path)?))
}

fn read() -> Option<Vec<Data>> {
	if let Ok(mut wtr) = csv::Reader::from_path(PASSWORD_FILE) {
		let vec = wtr.deserialize::<Data>().flatten().collect();
		return Some(vec);
	}

	if create_csv_file().is_ok() {
		return Some(Vec::new());
	}

	None
}

fn create_csv_file() -> Result<()> {
	let mut wtr = get_writer(PASSWORD_FILE.into())?;
	wtr.write_record(["Login", "Password", "Service"])?;
	wtr.flush()?;

	Ok(())
}

fn constraint_len_calculator(items: &[Data]) -> (u16, u16, u16) {
	let name_len = items
		.iter()
		.map(Data::login)
		.map(UnicodeWidthStr::width)
		.max()
		.unwrap_or(0);

	let password_len = items
		.iter()
		.map(Data::password)
		.map(UnicodeWidthStr::width)
		.max()
		.unwrap_or(0);

	let service_len = items
		.iter()
		.map(Data::service)
		.map(UnicodeWidthStr::width)
		.max()
		.unwrap_or(0);

	(name_len as u16, password_len as u16, service_len as u16)
}

#[allow(unused)]
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

#[derive(Clone, Debug, Default)]
pub struct UserInput {
	pub login: Input,
	pub password: Input,
	pub service: Input,
}

impl UserInput {
	pub fn login(&self) -> &str {
		self.login.value()
	}

	pub fn password(&self) -> &str {
		self.password.value()
	}

	pub fn service(&self) -> &str {
		self.service.value()
	}

	pub fn ref_array(&self) -> [&str; 3] {
		[self.login.value(), self.password.value(), self.service.value()]
	}

	pub fn reset_data(&mut self) {
		self.login.reset();
		self.password.reset();
		self.service.reset();
	}
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Data {
	pub login: String,
	pub password: String,
	pub service: String,
}

#[allow(unused)]
impl Data {
	fn new() -> Self {
		Data {
			login: String::new(),
			password: String::new(),
			service: String::new(),
		}
	}

	fn login(&self) -> &str {
		&self.login
	}

	fn password(&self) -> &str {
		&self.password
	}

	fn service(&self) -> &str {
		&self.service
	}

	pub fn ref_array(&self) -> [&str; 3] {
		[&self.login, &self.password, &self.service]
	}
}
