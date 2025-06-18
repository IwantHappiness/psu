use app::{App, CurrentScreen, InputMode, write};
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
					KeyCode::Esc => return Ok(true),
					KeyCode::Char('j') | KeyCode::Down => app.next_row(),
					KeyCode::Char('k') | KeyCode::Up => app.previous_row(),
					KeyCode::Char('l') | KeyCode::Right => app.nex_column(),
					KeyCode::Char('h') | KeyCode::Left => app.previous_column(),
					KeyCode::Char('n') => app.current_screen = CurrentScreen::Popup,
					_ => {}
				},
				CurrentScreen::Popup => match key.code {
					KeyCode::Esc => app.current_screen = CurrentScreen::Main,
					KeyCode::Enter => {
						// Skip print if fields are empty
						if app.input.login().is_empty()
							&& app.input.password().is_empty()
							&& app.input.service().is_empty()
						{
							continue;
						}

						write(app)?;
						app.input.reset_data();
						app.input_mode = InputMode::default();
						app.current_screen = CurrentScreen::default();
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
			}
		}
	}
}
