use color_eyre::Result;
use ratatui::{
	DefaultTerminal, Frame,
	crossterm::event::{self, Event, KeyCode, KeyEventKind},
	layout::{Constraint, Layout},
	style::{Color, Modifier, Style, Stylize},
	text::{Line, Text},
	widgets::{Block, Paragraph},
};
use serde::{Deserialize, Serialize};
use std::{fs, io};

pub struct App {
	data: Data,
	data_mode: DataMode,
	// character_index: usize,
	input_mode: InputMode,
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

	fn from(login: String, password: String, service: String) -> Self {
		Data {
			login,
			password,
			service,
		}
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
			// character_index: 0,
		}
	}

	fn print(&mut self) -> Result<(), io::Error> {
		if self.data.login.is_empty() && self.data.password.is_empty() && self.data.service.is_empty() {
			return Ok(());
		}

		let mut wtr = csv::WriterBuilder::new()
			.delimiter(b',')
			.quote_style(csv::QuoteStyle::NonNumeric)
			.from_writer(
				std::fs::OpenOptions::new()
					.write(true)
					.append(true)
					.create(true)
					.open("psu.csv")?,
			);

		if fs::metadata("psu.csv").map(|f| f.len() == 0).unwrap_or(true) {
			wtr.write_record(["login", "password", "service"])?;
		}

		wtr.write_record(self.data.ref_array())?;
		wtr.flush()?;

		self.data.login.clear();
		self.data.password.clear();
		self.data.service.clear();

		Ok(())
	}

	fn enter_char(&mut self, ch: char) {
		match self.data_mode {
			DataMode::Login => self.data.login.push(ch),
			DataMode::Password => self.data.password.push(ch),
			DataMode::Service => self.data.service.push(ch),
		}
	}

	fn delete_char(&mut self) {
		match self.data_mode {
			DataMode::Login => {
				self.data.login.pop();
			}
			DataMode::Password => {
				self.data.password.pop();
			}
			DataMode::Service => {
				self.data.service.pop();
			}
		}
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
						KeyCode::Enter => self.print()?,
						KeyCode::Backspace => self.delete_char(),
						KeyCode::Esc => self.input_mode = InputMode::Normal,
						KeyCode::Char(to_insert) => self.enter_char(to_insert),
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
		frame.render_widget(help_message, help_message_area);

		let login = Paragraph::new(self.data.login.as_str())
			.style(match self.input_mode {
				InputMode::Normal => Style::default(),
				InputMode::Editing => match self.data_mode {
					DataMode::Login => Style::default().fg(Color::Yellow),
					_ => Style::default(),
				},
			})
			.block(Block::bordered().title("Login"));
		frame.render_widget(login, login_area);

		let password = Paragraph::new(self.data.password.as_str())
			.style(match self.input_mode {
				InputMode::Normal => Style::default(),
				InputMode::Editing => match self.data_mode {
					DataMode::Password => Style::default().fg(Color::Yellow),
					_ => Style::default(),
				},
			})
			.block(Block::bordered().title("Password"));
		frame.render_widget(password, password_area);

		let service = Paragraph::new(self.data.service.as_str())
			.style(match self.input_mode {
				InputMode::Normal => Style::default(),
				InputMode::Editing => match self.data_mode {
					DataMode::Service => Style::default().fg(Color::Yellow),
					_ => Style::default(),
				},
			})
			.block(Block::bordered().title("Service"));
		frame.render_widget(service, service_area);
	}
}
