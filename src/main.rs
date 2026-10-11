#![warn(clippy::all, clippy::pedantic)]
// #![allow(unused)]

use app::App;
use color_eyre::Result;
use conf::Config;
use crossterm::{
	cursor,
	event::{DisableMouseCapture, EnableMouseCapture},
	execute,
	style::Stylize,
	terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, prelude::CrosstermBackend};
use run::run_app;
use std::{error::Error, io};

mod app;
mod conf;
mod run;
mod ui;

/// Undoes what `main` set up. Runs on normal exit, on `?` and while unwinding a panic.
struct TerminalGuard;

impl Drop for TerminalGuard {
	fn drop(&mut self) {
		restore_terminal();
	}
}

fn restore_terminal() {
	// Errors ignored: the guard also runs during unwinding, where there is nowhere to report to.
	let _ = disable_raw_mode();
	let _ = execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture);
	let _ = execute!(io::stderr(), cursor::Show);
}

fn install_panic_hook() {
	let previous = std::panic::take_hook();

	std::panic::set_hook(Box::new(move |info| {
		restore_terminal();
		previous(info);
	}));
}

fn main() -> Result<(), Box<dyn Error>> {
	let config = match Config::load() {
		Ok(c) => c,
		Err(e) => {
			eprintln!("{}", format!("ERROR: {e}").red());
			Config::new().unwrap_or_default()
		}
	};

	let mut app = App::new(config);
	let mut stderr = io::stderr();

	install_panic_hook();

	enable_raw_mode()?;
	execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
	let _guard = TerminalGuard;

	let backend = CrosstermBackend::new(stderr);
	let mut terminal = Terminal::new(backend)?;

	run_app(&mut terminal, &mut app)?;

	Ok(())
}
