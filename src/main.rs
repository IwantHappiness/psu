use app::App;
use color_eyre::Result;
use crossterm::{
	event::{DisableMouseCapture, EnableMouseCapture},
	execute,
	terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, prelude::CrosstermBackend};
use run::run_app;
use std::{error::Error, io};

mod app;
mod config;
mod run;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
	let mut app = App::new();

	enable_raw_mode()?;
	let mut stderr = io::stderr();
	execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

	let backend = CrosstermBackend::new(stderr);
	let mut terminal = Terminal::new(backend)?;
	let res = run_app(&mut terminal, &mut app);

	disable_raw_mode()?;
	execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
	terminal.show_cursor()?;

	if let Err(err) = res {
		eprintln!("Error: {:?}", err);
	}

	Ok(())
}
