use super::app::{App, CurrentScreen, ITEM_HEIGHT, InputMode};
use ratatui::{
	Frame,
	layout::{Constraint, Layout, Margin, Rect},
	style::{Color, Modifier, Style, Stylize},
	text::{Line, Text},
	widgets::{Block, Cell, HighlightSpacing, Paragraph, Row, Scrollbar, ScrollbarOrientation, Table},
};

const SCROLLBAR_BEGIN_SYMBOL: &str = "▲";
const SCROLLBAR_END_SYMBOL: &str = "▼";

pub fn ui(frame: &mut Frame, app: &mut App) {
	if app.current_screen == CurrentScreen::Table {
		// let vertical = Layout::vertical([Constraint::Min(5), Constraint::Length(4)]);
		let vertical = Layout::vertical([Constraint::Min(5)]);
		let rects = vertical.split(frame.area());
		render_table(app, frame, rects[0]);
		render_scrollbar(app, frame, rects[0]);
	} else {
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
}

fn render_table(app: &mut App, frame: &mut Frame, area: Rect) {
	let header_style = Style::default().fg(app.colors.header_fg).bg(app.colors.header_bg);
	let selected_row_style = Style::default()
		.add_modifier(Modifier::REVERSED)
		.fg(app.colors.selected_row_style_fg);
	let selected_col_style = Style::default().fg(app.colors.selected_column_style_fg);
	let selected_cell_style = Style::default()
		.add_modifier(Modifier::REVERSED)
		.fg(app.colors.selected_cell_style_fg);
	let header = ["Login", "Password", "Service"]
		.into_iter()
		.map(Cell::from)
		.collect::<Row>()
		.style(header_style)
		.height(1);

	let rows = app.items.iter().enumerate().map(|(i, data)| {
		let color = match i % 2 {
			0 => app.colors.normal_row_color,
			_ => app.colors.alt_row_color,
		};
		let item = data.ref_array();
		item.into_iter()
			.map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
			.collect::<Row>()
			.style(Style::new().fg(app.colors.row_fg).bg(color))
			.height(ITEM_HEIGHT as u16)
	});

	let bar = " █ ";
	let table = Table::new(
		rows,
		[
			// + 1 is for padding.
			Constraint::Min(app.longest_item_lens.0 + 1),
			Constraint::Min(app.longest_item_lens.1 + 1),
			Constraint::Min(app.longest_item_lens.2),
		],
	)
	.header(header)
	.row_highlight_style(selected_row_style)
	.column_highlight_style(selected_col_style)
	.cell_highlight_style(selected_cell_style)
	.highlight_symbol(Text::from(vec!["".into(), bar.into(), "".into()]))
	.bg(app.colors.buffer_bg)
	.highlight_spacing(HighlightSpacing::Always);
	frame.render_stateful_widget(table, area, &mut app.state);
}

fn render_scrollbar(app: &mut App, frame: &mut Frame, area: Rect) {
	frame.render_stateful_widget(
		Scrollbar::default()
			.orientation(ScrollbarOrientation::VerticalRight)
			.begin_symbol(Some(SCROLLBAR_BEGIN_SYMBOL))
			.end_symbol(Some(SCROLLBAR_END_SYMBOL)),
		area.inner(Margin {
			vertical: 1,
			horizontal: 1,
		}),
		&mut app.scroll_state,
	);
}

fn render_input(app: &App, frame: &mut Frame, area: (Rect, Rect, Rect)) {
	let login = Paragraph::new(app.input.login())
		.style(match app.current_screen {
			CurrentScreen::Editing if app.input_mode == InputMode::Login => Style::default().fg(Color::White),
			_ => Style::default(),
		})
		.block(Block::bordered().title("Login or Email"));
	let password = Paragraph::new(app.input.password())
		.style(match app.current_screen {
			CurrentScreen::Editing if app.input_mode == InputMode::Password => Style::default().fg(Color::Blue),
			_ => Style::default(),
		})
		.block(Block::bordered().title("Password"));
	let service = Paragraph::new(app.input.service())
		.style(match app.current_screen {
			CurrentScreen::Editing if app.input_mode == InputMode::Service => Style::default().fg(Color::Red),
			_ => Style::default(),
		})
		.block(Block::bordered().title("Service"));

	frame.render_widget(login, area.0);
	frame.render_widget(password, area.1);
	frame.render_widget(service, area.2);

	let width = match app.input_mode {
		InputMode::Login => area.0.width.max(3) - 3,
		InputMode::Password => area.1.width.max(3) - 3,
		InputMode::Service => area.2.width.max(3) - 3,
	};

	let scroll = match app.input_mode {
		InputMode::Login => app.input.login.visual_scroll(width as usize),
		InputMode::Password => app.input.password.visual_scroll(width as usize),
		InputMode::Service => app.input.service.visual_scroll(width as usize),
	};

	if app.current_screen == CurrentScreen::Editing {
		let x = match app.input_mode {
			InputMode::Login => app.input.login.visual_cursor().max(scroll) - scroll + 1,
			InputMode::Password => app.input.password.visual_cursor().max(scroll) - scroll + 1,
			InputMode::Service => app.input.service.visual_cursor().max(scroll) - scroll + 1,
		};
		match app.input_mode {
			InputMode::Login => frame.set_cursor_position((area.0.x + x as u16, area.0.y + 1)),
			InputMode::Password => frame.set_cursor_position((area.1.x + x as u16, area.1.y + 1)),
			InputMode::Service => frame.set_cursor_position((area.2.x + x as u16, area.2.y + 1)),
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
				"q or n".bold(),
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
		CurrentScreen::Table => (vec![], Style::default()),
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
