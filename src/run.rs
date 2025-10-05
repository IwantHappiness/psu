use super::app::{App, CurrentScreen, Data, InputMode};
use super::ui::ui;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{Terminal, prelude::Backend};
use std::error::Error;
use tui_input::backend::crossterm::EventHandler;

pub fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> anyhow::Result<bool, Box<dyn Error>> {
	loop {
		terminal.draw(|frame| ui(frame, app))?;

		let event = event::read()?;
		if let Event::Key(key) = event {
			if key.kind == event::KeyEventKind::Release {
				continue;
			}

			match app.current_screen {
				CurrentScreen::Main => match key.code {
					KeyCode::Esc => return Ok(true),
					KeyCode::Char('P') => app.clip_row()?,
					KeyCode::Char('c') => app.clip_column()?,
					KeyCode::Char('p') => app.clip_password()?,
					KeyCode::Char('j') | KeyCode::Down => app.next_row(),
					KeyCode::Char('k') | KeyCode::Up => app.previous_row(),
					KeyCode::Char('l') | KeyCode::Right => app.nex_column(),
					KeyCode::Char('h') | KeyCode::Left => app.previous_column(),
					KeyCode::Char('d' | 'D') => {
						app.delete();
						app.write()?;
					}
					KeyCode::Char('?') => app.current_screen = CurrentScreen::Help,
					KeyCode::Char('n' | 'N') => app.current_screen = CurrentScreen::Popup,
					KeyCode::Char('m' | 'M') => {
						app.modify();
						app.current_screen = CurrentScreen::Popup;
					}
					_ => {}
				},
				CurrentScreen::Popup => match key.code {
					KeyCode::Esc => {
						if app.is_modify {
							app.input.reset_data();
							app.is_modify = false;
						}
						app.current_screen = CurrentScreen::Main;
					}
					KeyCode::Enter => {
						// Skip print if fields are empty
						if app.input.login().is_empty()
							&& app.input.password().is_empty()
							&& app.input.service().is_empty()
						{
							continue;
						}

						app.add_password();
						app.write()?;
						app.input.reset_data();
						app.input_mode = InputMode::default();
						app.current_screen = CurrentScreen::Main;
					}
					// Switch fields
					KeyCode::Down | KeyCode::Tab => app.next_input_mode(),
					KeyCode::Up => app.prev_input_mode(),
					_ => {
						// Switch inputs for fields
						match app.input_mode {
							InputMode::Login => app.input.login.handle_event(&event),
							InputMode::Password => app.input.password.handle_event(&event),
							InputMode::Service => app.input.service.handle_event(&event),
						};
					}
				},
				CurrentScreen::Help => match key.code {
					KeyCode::Esc => app.current_screen = CurrentScreen::Main,
					_ => {}
				},
			}
		}
	}
}
