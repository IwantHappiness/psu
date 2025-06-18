use crate::app::CurrentScreen;

use super::app::{App, ITEM_HEIGHT, InputMode};
use color_eyre::owo_colors::OwoColorize;
use derive_setters::Setters;
use ratatui::{
	Frame,
	buffer::Buffer,
	layout::{Constraint, Direction, Layout, Margin, Rect},
	style::{Color, Modifier, Style, Stylize},
	text::{Line, Text},
	widgets::{
		Block, Borders, Cell, Clear, HighlightSpacing, Paragraph, Row, Scrollbar, ScrollbarOrientation, Table, Widget,
		Wrap,
	},
};

const SCROLLBAR_BEGIN_SYMBOL: &str = "▲";
const SCROLLBAR_END_SYMBOL: &str = "▼";

#[allow(unused)]
#[derive(Debug, Default, Setters)]
struct Popup<'a> {
	#[setters(into)]
	title: Line<'a>,
	#[setters(into)]
	content: Text<'a>,
	border_style: Style,
	title_style: Style,
	style: Style,
}

// TODO: idk
// impl Widget for Popup<'_> {
// 	fn render(self, area: Rect, buf: &mut Buffer) {
// 		Clear.render(area, buf);
// 		let block = Block::new()
// 			.title(self.title)
// 			.title_style(self.title_style)
// 			.borders(Borders::ALL)
// 			.border_style(self.border_style);
// 		Paragraph::new(self.content)
// 			.wrap(Wrap { trim: true })
// 			.style(self.style)
// 			.block(block)
// 			.render(area, buf);
// 	}
// }

pub fn ui(frame: &mut Frame, app: &mut App) {
	// let vertical = Layout::vertical([Constraint::Min(5), Constraint::Length(4)]);
	let vertical = Layout::vertical([Constraint::Min(5)]);
	let rects = vertical.split(frame.area());
	if matches!(app.current_screen, CurrentScreen::Popup) {
		render_popup(app, frame);
	} else {
		render_table(app, frame, rects[0]);
		render_scrollbar(app, frame, rects[0]);
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

pub fn render_popup(app: &App, frame: &mut Frame) {
	let popup_block = Block::default()
		.title("Form")
		.borders(Borders::ALL)
		.style(Style::default());
	let area = centered_rect(60, 40, frame.area());

	frame.render_widget(popup_block, area);

	let popup_chunks = Layout::default()
		.direction(Direction::Vertical)
		.margin(1)
		.constraints([
			Constraint::Percentage(50),
			Constraint::Percentage(50),
			Constraint::Percentage(50),
		])
		.split(area);

	let mut login_block = Block::default()
		.title("Login or Email")
		// .bg(Color::Black)
		.borders(Borders::ALL);
	let mut password_block = Block::default()
		.title("Password")
		// .bg(Color::Black)
		.borders(Borders::ALL);
	let mut service_block = Block::default()
		.title("Service")
		// .bg(Color::Black)
		.borders(Borders::ALL);

	let active_style = Style::default().fg(Color::Red);

	match app.input_mode {
		InputMode::Login => login_block = login_block.style(active_style),
		InputMode::Password => password_block = password_block.style(active_style),
		InputMode::Service => service_block = service_block.style(active_style),
	};

	let login_text = Paragraph::new(app.input.login()).block(login_block).fg(Color::White);
	frame.render_widget(login_text, popup_chunks[0]);

	let password = Paragraph::new(app.input.password()).block(password_block);
	frame.render_widget(password, popup_chunks[1]);

	let service_text = Paragraph::new(app.input.service()).block(service_block);
	frame.render_widget(service_text, popup_chunks[2]);

	let width = match app.input_mode {
		InputMode::Login => popup_chunks[0].width.max(3) - 3,
		InputMode::Password => popup_chunks[1].width.max(3) - 3,
		InputMode::Service => popup_chunks[2].width.max(3) - 3,
	};

	let scroll = match app.input_mode {
		InputMode::Login => app.input.login.visual_scroll(width as usize),
		InputMode::Password => app.input.password.visual_scroll(width as usize),
		InputMode::Service => app.input.service.visual_scroll(width as usize),
	};

	let x = match app.input_mode {
		InputMode::Login => app.input.login.visual_cursor().max(scroll) - scroll + 1,
		InputMode::Password => app.input.password.visual_cursor().max(scroll) - scroll + 1,
		InputMode::Service => app.input.service.visual_cursor().max(scroll) - scroll + 1,
	};

	match app.input_mode {
		InputMode::Login => frame.set_cursor_position((popup_chunks[0].x + x as u16, popup_chunks[0].y + 1)),
		InputMode::Password => frame.set_cursor_position((popup_chunks[1].x + x as u16, popup_chunks[1].y + 1)),
		InputMode::Service => frame.set_cursor_position((popup_chunks[2].x + x as u16, popup_chunks[2].y + 1)),
	};

	// let popup = Popup::default()
	// 	.content("Hello world!")
	// 	.style(Style::new().yellow())
	// 	.title("With Clear")
	// 	.title_style(Style::new().white().bold())
	// 	.border_style(Style::new().red());
	// frame.render_widget(popup, area);
}

// #[allow(unused)]
// fn render_help_message(app: &App, frame: &mut Frame, area: Rect) {
// 	let (msg, style) = match app.current_screen {
// 		CurrentScreen::Main => (
// 			vec![
// 				"Press ".into(),
// 				"q".bold(),
// 				" to exit, ".into(),
// 				"e".bold(),
// 				" to start editing.".bold(),
// 			],
// 			Style::default().add_modifier(Modifier::RAPID_BLINK),
// 		),
// 		CurrentScreen::Exiting => (
// 			vec![
// 				"Press ".into(),
// 				"y".bold(),
// 				" to exit psu, ".into(),
// 				"q or n".bold(),
// 				" to cancel exit.".into(),
// 			],
// 			Style::default().add_modifier(Modifier::RAPID_BLINK),
// 		),
// 		CurrentScreen::Editing => (
// 			vec![
// 				"Press ".into(),
// 				"Esc".bold(),
// 				" to stop editing, ".into(),
// 				"Tab and Arrows".bold(),
// 				" to switch fields, ".into(),
// 				"Enter".bold(),
// 				" to record the password.".into(),
// 			],
// 			Style::default()
// 				.add_modifier(Modifier::ITALIC)
// 				.add_modifier(Modifier::RAPID_BLINK),
// 		),
// 		CurrentScreen::Popup => (vec![], Style::default()),
// 	};

// 	let text = Text::from(Line::from(msg)).patch_style(style);
// 	let help_message = Paragraph::new(text);
// 	frame.render_widget(help_message, area);
// }

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
	let popup_layout = Layout::default()
		.direction(Direction::Vertical)
		.constraints([
			Constraint::Percentage((100 - percent_y) / 2),
			Constraint::Percentage(percent_y),
			Constraint::Percentage((100 - percent_y) / 2),
		])
		.split(r);

	Layout::default()
		.direction(Direction::Horizontal)
		.constraints([
			Constraint::Percentage((100 - percent_x) / 2),
			Constraint::Percentage(percent_x),
			Constraint::Percentage((100 - percent_x) / 2),
		])
		.split(popup_layout[1])[1] // Return the middle chunk
}
