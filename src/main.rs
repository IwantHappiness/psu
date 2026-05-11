#![warn(clippy::all, clippy::pedantic)]
// #![allow(unused)]
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
mod conf;
mod run;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
	let mut app = App::new();

	enable_raw_mode().unwrap_or_else(|e| {
		eprintln!("Error enabling raw mode: {e:?}");
		std::process::exit(1);
	});

	let mut stderr = io::stderr();
	if let Err(e) = execute!(stderr, EnterAlternateScreen, EnableMouseCapture) {
		eprintln!("Error: {e}");
		std::process::exit(1);
	}

	let backend = CrosstermBackend::new(stderr);
	let mut terminal = Terminal::new(backend).expect("Failed to create terminal");
	run_app(&mut terminal, &mut app).unwrap_or_else(|e| {
		eprintln!("Error: {e:?}");
		std::process::exit(1);
	});

	if let Err(e) = disable_raw_mode() {
		eprintln!("Error disabling raw mode: {e:?}");
		app.write().unwrap_or_else(|e| eprintln!("Error: {e}"));
		std::process::exit(1);
	}
	execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
	terminal.show_cursor().unwrap_or_else(|e| {
		eprintln!("Error: {e:?}");
	});

	Ok(())
}
