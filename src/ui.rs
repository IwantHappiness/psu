use crate::app::{App, CurrentScreen, DataMode};
use ratatui::{
	Frame,
	layout::{Constraint, Layout, Rect},
	style::{Color, Modifier, Style, Stylize},
	text::{Line, Text},
	widgets::{Block, Paragraph},
};

pub fn ui(frame: &mut Frame, app: &App) {
	let vertical = Layout::vertical([
		Constraint::Length(3),
		Constraint::Length(3),
		Constraint::Length(3),
		Constraint::Length(1),
	]);
	let [login_area, password_area, service_area, help_message_area] = vertical.areas(frame.area());

	// Render fileds
	render_input(app, frame, (login_area, password_area, service_area));
	render_help_message(app, frame, help_message_area);
}

fn render_input(app: &App, frame: &mut Frame, area: (Rect, Rect, Rect)) {
	let login = Paragraph::new(app.data.login.value())
		.style(match app.current_screen {
			CurrentScreen::Editing => match app.data_mode {
				DataMode::Login => Style::default().fg(Color::White),
				_ => Style::default(),
			},
			_ => Style::default(),
		})
		.block(Block::bordered().title("Login or Email"));
	let password = Paragraph::new(app.data.password.value())
		.style(match app.current_screen {
			CurrentScreen::Editing => match app.data_mode {
				DataMode::Password => Style::default().fg(Color::Blue),
				_ => Style::default(),
			},
			_ => Style::default(),
		})
		.block(Block::bordered().title("Password"));
	let service = Paragraph::new(app.data.service.value())
		.style(match app.current_screen {
			CurrentScreen::Editing => match app.data_mode {
				DataMode::Service => Style::default().fg(Color::Red),
				_ => Style::default(),
			},
			_ => Style::default(),
		})
		.block(Block::bordered().title("Service"));

	frame.render_widget(login, area.0);
	frame.render_widget(password, area.1);
	frame.render_widget(service, area.2);

	let width = match app.data_mode {
		DataMode::Login => area.0.width.max(3) - 3,
		DataMode::Password => area.1.width.max(3) - 3,
		DataMode::Service => area.2.width.max(3) - 3,
	};

	let scroll = match app.data_mode {
		DataMode::Login => app.data.login.visual_scroll(width as usize),
		DataMode::Password => app.data.password.visual_scroll(width as usize),
		DataMode::Service => app.data.service.visual_scroll(width as usize),
	};

	if app.current_screen == CurrentScreen::Editing {
		let x = match app.data_mode {
			DataMode::Login => app.data.login.visual_cursor().max(scroll) - scroll + 1,
			DataMode::Password => app.data.password.visual_cursor().max(scroll) - scroll + 1,
			DataMode::Service => app.data.service.visual_cursor().max(scroll) - scroll + 1,
		};
		match app.data_mode {
			DataMode::Login => frame.set_cursor_position((area.0.x + x as u16, area.0.y + 1)),
			DataMode::Password => frame.set_cursor_position((area.1.x + x as u16, area.1.y + 1)),
			DataMode::Service => frame.set_cursor_position((area.2.x + x as u16, area.2.y + 1)),
		};
	}
}

fn render_help_message(app: &App, frame: &mut Frame, area: Rect) {
	let (msg, style) = match app.current_screen {
		CurrentScreen::Main => (
			vec![
				"Press ".into(),
				"q".bold(),
				" to exit, ".into(),
				"e".bold(),
				" to start editing.".bold(),
			],
			Style::default().add_modifier(Modifier::RAPID_BLINK),
		),
		CurrentScreen::Exiting => (
			vec![
				"Press ".into(),
				"y".bold(),
				" to exit psu, ".into(),
				"q".bold(),
				" to cancel exit.".into(),
			],
			Style::default().add_modifier(Modifier::RAPID_BLINK),
		),
		CurrentScreen::Editing => (
			vec![
				"Press ".into(),
				"Esc".bold(),
				" to stop editing, ".into(),
				"Tab and Arrows".bold(),
				" to switch fields, ".into(),
				"Enter".bold(),
				" to record the password.".into(),
			],
			Style::default()
				.add_modifier(Modifier::ITALIC)
				.add_modifier(Modifier::RAPID_BLINK),
		),
	};

	let text = Text::from(Line::from(msg)).patch_style(style);
	let help_message = Paragraph::new(text);
	frame.render_widget(help_message, area);
}

// TODO: may be needed
//
// #[allow(unused)]
// fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
// 	let popup_layout = Layout::default()
// 		.direction(Direction::Vertical)
// 		.constraints([
// 			Constraint::Percentage((100 - percent_y) / 2),
// 			Constraint::Percentage(percent_y),
// 			Constraint::Percentage((100 - percent_y) / 2),
// 		])
// 		.split(r);

// 	Layout::default()
// 		.direction(Direction::Horizontal)
// 		.constraints([
// 			Constraint::Percentage((100 - percent_x) / 2),
// 			Constraint::Percentage(percent_x),
// 			Constraint::Percentage((100 - percent_x) / 2),
// 		])
// 		.split(popup_layout[1])[1] // Return the middle chunk
// }
