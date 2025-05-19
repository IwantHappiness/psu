use app::{App, CurrentScreen, DataMode};
use color_eyre::Result;
use config::Config;
use crossterm::{
	event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
	execute,
	terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
	Terminal,
	prelude::{Backend, CrosstermBackend},
};
use std::{error::Error, io};
use tui_input::backend::crossterm::EventHandler;
use ui::ui;

mod app;
mod config;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
	let conf = Config::new();

	if let Err(e) = conf.generate_conf() {
		eprintln!("Error: {e}");
	}

	enable_raw_mode()?;
	let mut stderr = io::stderr();
	execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

	let backend = CrosstermBackend::new(stderr);
	let mut terminal = Terminal::new(backend)?;

	let mut app = App::new();
	let res = run_app(&mut terminal, &mut app);

	disable_raw_mode()?;
	execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
	terminal.show_cursor()?;

	if let Err(err) = res {
		println!("{:?}", err);
	}

	Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> anyhow::Result<bool> {
	loop {
		terminal.draw(|frame| ui(frame, app))?;

		let event = event::read()?;
		if let Event::Key(key) = event {
			if key.kind == event::KeyEventKind::Release {
				continue;
			}

			match app.current_screen {
				CurrentScreen::Main => match key.code {
					KeyCode::Char('e') => app.current_screen = CurrentScreen::Editing,
					KeyCode::Char('q') => app.current_screen = CurrentScreen::Exiting,
					_ => {}
				},

				CurrentScreen::Exiting => match key.code {
					KeyCode::Char('y') => return Ok(true),
					KeyCode::Char('n') | KeyCode::Char('q') => app.current_screen = CurrentScreen::Main,
					_ => {}
				},
				CurrentScreen::Editing => match key.code {
					// Print password to PASSWORD_FILE
					KeyCode::Enter => {
						// Skip print if fields are empty
						if app.data.login.value().is_empty()
							&& app.data.password.value().is_empty()
							&& app.data.service.value().is_empty()
						{
							continue;
						}

						app.print()?;
						app.data.reset_data();
						app.data_mode = DataMode::default();
						app.current_screen = CurrentScreen::default();
					}
					KeyCode::Esc => app.current_screen = CurrentScreen::Main,
					// Switch fields
					KeyCode::Down | KeyCode::Tab => match app.data_mode {
						DataMode::Login => app.data_mode = DataMode::Password,
						DataMode::Password => app.data_mode = DataMode::Service,
						DataMode::Service => app.data_mode = DataMode::Login,
					},
					KeyCode::Up => match app.data_mode {
						DataMode::Login => app.data_mode = DataMode::Service,
						DataMode::Password => app.data_mode = DataMode::Login,
						DataMode::Service => app.data_mode = DataMode::Password,
					},
					// Switch inputs for fields
					_ => {
						match app.data_mode {
							DataMode::Login => app.data.login.handle_event(&event),
							DataMode::Password => app.data.password.handle_event(&event),
							DataMode::Service => app.data.service.handle_event(&event),
						};
					}
				},
			}
		}
	}
}
