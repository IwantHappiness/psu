use color_eyre::Result;
use ratatui::{
	DefaultTerminal, Frame,
	crossterm::event::{self, Event, KeyCode, KeyEventKind},
	layout::{Constraint, Layout, Position},
	style::{Color, Modifier, Style, Stylize},
	text::{Line, Text},
	widgets::{Block, Paragraph},
};
use serde::{Deserialize, Serialize};
use std::{fs, io};

const PASSWORD_FILE: &str = "psu.csv";

pub struct App {
	// Data form
	data: Data,
	// Current field
	data_mode: DataMode,
	// Current input mode
	input_mode: InputMode,

	character_index_login: usize,
	character_index_password: usize,
	character_index_service: usize,
}

enum DataMode {
	Login,
	Password,
	Service,
}

enum InputMode {
	Normal,
	Editing,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Data {
	pub login: String,
	pub password: String,
	pub service: String,
}

impl Data {
	fn new() -> Self {
		Data {
			login: String::new(),
			password: String::new(),
			service: String::new(),
		}
	}

	#[allow(unused)]
	fn from(login: String, password: String, service: String) -> Self {
		Data {
			login,
			password,
			service,
		}
	}

	fn reset_data(&mut self) {
		self.login.clear();
		self.password.clear();
		self.service.clear();
	}

	fn ref_array(&self) -> [&str; 3] {
		[&self.login, &self.password, &self.service]
	}
}

impl App {
	pub fn new() -> Self {
		Self {
			data: Data::new(),
			data_mode: DataMode::Login,
			input_mode: InputMode::Normal,
			character_index_login: 0,
			character_index_password: 0,
			character_index_service: 0,
		}
	}

	// Print password to PASSWORD_FILE
	fn print(&mut self) -> Result<(), io::Error> {
		if self.data.login.is_empty() && self.data.password.is_empty() && self.data.service.is_empty() {
			return Ok(());
		}

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
			wtr.write_record(["login", "password", "service"])?;
		}

		wtr.write_record(self.data.ref_array())?;
		wtr.flush()?;

		self.data.reset_data();
		self.reset_cursor();

		Ok(())
	}

	fn enter_char(&mut self, ch: char) {
		match self.data_mode {
			DataMode::Login => {
				let index = self.byte_index();
				self.data.login.insert(index, ch);
				self.move_cursor_right();
			}
			DataMode::Password => {
				let index = self.byte_index();
				self.data.password.insert(index, ch);
				self.move_cursor_right();
			}
			DataMode::Service => {
				let index = self.byte_index();
				self.data.service.insert(index, ch);
				self.move_cursor_right();
			}
		}
	}

	fn delete_char(&mut self) {
		match self.data_mode {
			DataMode::Login => {
				if self.character_index_login != 0 {
					let current_index = self.character_index_login;
					let from_left_to_current_index = current_index - 1;
					let before_char_to_delete = self.data.login.chars().take(from_left_to_current_index);
					let after_char_to_delete = self.data.login.chars().skip(current_index);
					self.data.login = before_char_to_delete.chain(after_char_to_delete).collect();
					self.move_cursor_left();
				}
			}
			DataMode::Password => {
				if self.character_index_password != 0 {
					let current_index = self.character_index_password;
					let from_left_to_current_index = current_index - 1;
					let before_char_to_delete = self.data.password.chars().take(from_left_to_current_index);
					let after_char_to_delete = self.data.password.chars().skip(current_index);
					self.data.password = before_char_to_delete.chain(after_char_to_delete).collect();
					self.move_cursor_left();
				}
			}
			DataMode::Service => {
				if self.character_index_service != 0 {
					let current_index = self.character_index_service;
					let from_left_to_current_index = current_index - 1;
					let before_char_to_delete = self.data.service.chars().take(from_left_to_current_index);
					let after_char_to_delete = self.data.service.chars().skip(current_index);
					self.data.service = before_char_to_delete.chain(after_char_to_delete).collect();
					self.move_cursor_left();
				}
			}
		}
	}

	fn byte_index(&self) -> usize {
		match self.data_mode {
			DataMode::Login => self
				.data
				.login
				.char_indices()
				.map(|(i, _)| i)
				.nth(self.character_index_login)
				.unwrap_or(self.data.login.len()),
			DataMode::Password => self
				.data
				.password
				.char_indices()
				.map(|(i, _)| i)
				.nth(self.character_index_password)
				.unwrap_or(self.data.password.len()),
			DataMode::Service => self
				.data
				.service
				.char_indices()
				.map(|(i, _)| i)
				.nth(self.character_index_service)
				.unwrap_or(self.data.service.len()),
		}
	}

	fn move_cursor_right(&mut self) {
		match self.data_mode {
			DataMode::Login => {
				let cursor_moved_right = self.character_index_login.saturating_add(1);
				self.character_index_login = self.clamp_cursor(cursor_moved_right);
			}
			DataMode::Password => {
				let cursor_moved_right = self.character_index_password.saturating_add(1);
				self.character_index_password = self.clamp_cursor(cursor_moved_right);
			}
			DataMode::Service => {
				let cursor_moved_right = self.character_index_service.saturating_add(1);
				self.character_index_service = self.clamp_cursor(cursor_moved_right);
			}
		}
	}

	fn move_cursor_left(&mut self) {
		match self.data_mode {
			DataMode::Login => {
				let cursor_moved_right = self.character_index_login.saturating_sub(1);
				self.character_index_login = self.clamp_cursor(cursor_moved_right);
			}
			DataMode::Password => {
				let cursor_moved_right = self.character_index_password.saturating_sub(1);
				self.character_index_password = self.clamp_cursor(cursor_moved_right);
			}
			DataMode::Service => {
				let cursor_moved_right = self.character_index_service.saturating_sub(1);
				self.character_index_service = self.clamp_cursor(cursor_moved_right);
			}
		}
	}

	fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
		match self.data_mode {
			DataMode::Login => new_cursor_pos.clamp(0, self.data.login.chars().count()),
			DataMode::Password => new_cursor_pos.clamp(0, self.data.password.chars().count()),
			DataMode::Service => new_cursor_pos.clamp(0, self.data.service.chars().count()),
		}
	}

	fn reset_cursor(&mut self) {
		self.character_index_login = 0;
		self.character_index_password = 0;
		self.character_index_service = 0;
	}

	pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
		loop {
			terminal.draw(|frame| self.draw(frame))?;

			if let Event::Key(key) = event::read()? {
				match self.input_mode {
					InputMode::Normal => match key.code {
						KeyCode::Char('q') => return Ok(()),
						KeyCode::Char('e') => self.input_mode = InputMode::Editing,
						_ => {}
					},

					InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
						// Print password to PASSWORD_FILE
						KeyCode::Enter => self.print()?,
						KeyCode::Backspace => self.delete_char(),
						KeyCode::Esc => self.input_mode = InputMode::Normal,
						KeyCode::Char(to_insert) => self.enter_char(to_insert),
						// Switch fields
						KeyCode::Down | KeyCode::Tab => match self.data_mode {
							DataMode::Login => self.data_mode = DataMode::Password,
							DataMode::Password => self.data_mode = DataMode::Service,
							DataMode::Service => self.data_mode = DataMode::Login,
						},
						KeyCode::Up => match self.data_mode {
							DataMode::Login => self.data_mode = DataMode::Service,
							DataMode::Password => self.data_mode = DataMode::Login,
							DataMode::Service => self.data_mode = DataMode::Password,
						},
						// Switch cursor in fields
						KeyCode::Left => self.move_cursor_left(),
						KeyCode::Right => self.move_cursor_right(),
						_ => {}
					},
					InputMode::Editing => {}
				}
			}
		}
	}

	fn draw(&self, frame: &mut Frame) {
		let vertical = Layout::vertical([
			Constraint::Length(3),
			Constraint::Length(3),
			Constraint::Length(3),
			Constraint::Length(1),
		]);
		let [login_area, password_area, service_area, help_message_area] = vertical.areas(frame.area());

		let (msg, style) = match self.input_mode {
			InputMode::Normal => (
				vec![
					"Press ".into(),
					"q".bold(),
					" to exit, ".into(),
					"e".bold(),
					" to start editing.".bold(),
				],
				Style::default().add_modifier(Modifier::RAPID_BLINK),
			),
			InputMode::Editing => (
				vec![
					"Press ".into(),
					"Esc".bold(),
					" to stop editing, ".into(),
					"Tab and Arrows".bold(),
					" to switch fields, ".into(),
					"Enter".bold(),
					" to record the password".into(),
				],
				Style::default(),
			),
		};
		let text = Text::from(Line::from(msg)).patch_style(style);
		let help_message = Paragraph::new(text);
		let login = Paragraph::new(self.data.login.as_str())
			.style(match self.input_mode {
				InputMode::Normal => Style::default(),
				InputMode::Editing => match self.data_mode {
					DataMode::Login => Style::default().fg(Color::Yellow),
					_ => Style::default(),
				},
			})
			.block(Block::bordered().title("Login"));
		let password = Paragraph::new(self.data.password.as_str())
			.style(match self.input_mode {
				InputMode::Normal => Style::default(),
				InputMode::Editing => match self.data_mode {
					DataMode::Password => Style::default().fg(Color::Yellow),
					_ => Style::default(),
				},
			})
			.block(Block::bordered().title("Password"));
		let service = Paragraph::new(self.data.service.as_str())
			.style(match self.input_mode {
				InputMode::Normal => Style::default(),
				InputMode::Editing => match self.data_mode {
					DataMode::Service => Style::default().fg(Color::Yellow),
					_ => Style::default(),
				},
			})
			.block(Block::bordered().title("Service"));

		// Render fileds
		frame.render_widget(help_message, help_message_area);
		frame.render_widget(login, login_area);
		frame.render_widget(password, password_area);
		frame.render_widget(service, service_area);

		// Cursor handling for input fields
		match self.data_mode {
			DataMode::Login => {
				if let InputMode::Editing = self.input_mode {
					frame.set_cursor_position(Position::new(
						login_area.x + self.character_index_login as u16 + 1,
						login_area.y + 1,
					))
				}
			}
			DataMode::Password => {
				if let InputMode::Editing = self.input_mode {
					frame.set_cursor_position(Position::new(
						password_area.x + self.character_index_password as u16 + 1,
						password_area.y + 1,
					))
				}
			}
			DataMode::Service => {
				if let InputMode::Editing = self.input_mode {
					frame.set_cursor_position(Position::new(
						service_area.x + self.character_index_service as u16 + 1,
						service_area.y + 1,
					))
				}
			}
		}
	}
}
