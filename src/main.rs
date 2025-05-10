use app::App;
use color_eyre::Result;
use config::Config;

mod app;
mod config;

fn main() -> Result<()> {
	if let Err(e) = Config::new().generate_conf() {
		eprintln!("Error: {e}");
	}

	color_eyre::install()?;
	let terminal = ratatui::init();
	let app_result = App::new().run(terminal);
	ratatui::restore();
	app_result
}
