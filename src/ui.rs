use super::app::{App, CurrentScreen, Data, ITEM_HEIGHT, InputMode};
use ratatui::{
	Frame,
	layout::{Constraint, Direction, Layout, Margin, Rect},
	style::{Color, Modifier, Style, Stylize, palette::tailwind},
	text::Text,
	widgets::{
		Block, BorderType, Borders, Cell, Clear, HighlightSpacing, Paragraph, Row, Scrollbar, ScrollbarOrientation,
		Table,
	},
};
use unicode_width::UnicodeWidthStr;

const SCROLLBAR_BEGIN_SYMBOL: &str = "▲";
const SCROLLBAR_END_SYMBOL: &str = "▼";
const INFO_TEXT: [&str; 2] = [
	"(Esc) quit | (↑) move up | (↓) move down | (←) move left | (→) move right",
	"(N) new password | (Enter) send password | (D) delete password | (M) modify password",
];

#[derive(Default)]
pub struct TableColors {
	pub alt_row_color: Color,
	pub buffer_bg: Color,
	pub footer_border_color: Color,
	pub header_bg: Color,
	pub header_fg: Color,
	pub normal_row_color: Color,
	pub row_fg: Color,
	pub selected_cell_style_fg: Color,
	pub selected_column_style_fg: Color,
	pub selected_row_style_fg: Color,
}

impl TableColors {
	pub const fn new(color: &tailwind::Palette) -> Self {
		Self {
			buffer_bg: Color::Reset,
			header_bg: Color::Reset,
			header_fg: tailwind::SLATE.c200,
			row_fg: tailwind::SLATE.c50,
			selected_row_style_fg: color.c400,
			selected_column_style_fg: color.c400,
			selected_cell_style_fg: color.c600,
			normal_row_color: Color::Reset,
			alt_row_color: Color::Reset,
			footer_border_color: color.c50,
		}
	}
}

pub fn ui(frame: &mut Frame, app: &mut App) {
	let vertical = Layout::vertical([Constraint::Min(5), Constraint::Length(4)]);
	let rects = vertical.split(frame.area());

	render_table(app, frame, rects[0]);
	render_scrollbar(app, frame, rects[0]);
	render_footer(app, frame, rects[1]);

	if app.current_screen == CurrentScreen::Popup {
		render_popup(app, frame);
	}
}

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
		data.ref_array()
			.into_iter()
			.skip(1)
			.map(|content| Cell::from(Text::from(format!("\n{content}\n"))))
			.collect::<Row>()
			.style(Style::new().fg(app.colors.row_fg).bg(color))
			.height(ITEM_HEIGHT as u16)
	});

	let bar = " █ ";
	let longest_item_lens = constraint_len_calculator(&app.items);
	let table = Table::new(
		rows,
		[
			Constraint::Min(longest_item_lens.0 + 1),
			Constraint::Min(longest_item_lens.1 + 1),
			Constraint::Min(longest_item_lens.2),
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
		.style(Style::default())
		.bg(Color::Black);
	let area = centered_rect(60, 37, frame.area());

	frame.render_widget(Clear, area);
	frame.render_widget(popup_block, area);

	let popup_chunks = Layout::default()
		.direction(Direction::Vertical)
		.margin(1)
		.constraints([Constraint::Fill(1), Constraint::Fill(1), Constraint::Fill(1)])
		.split(area);

	let mut login_block = Block::default().title("Login or Email").borders(Borders::ALL);
	let mut password_block = Block::default().title("Password").borders(Borders::ALL);
	let mut service_block = Block::default().title("Service").borders(Borders::ALL);

	let active_style = Style::default().fg(Color::Red);

	match app.input_mode {
		InputMode::Login => login_block = login_block.style(active_style),
		InputMode::Password => password_block = password_block.style(active_style),
		InputMode::Service => service_block = service_block.style(active_style),
	};

	let login_text = Paragraph::new(app.input.login()).fg(Color::White).block(login_block);
	frame.render_widget(login_text, popup_chunks[0]);

	let password = Paragraph::new(app.input.password())
		.fg(Color::White)
		.block(password_block);
	frame.render_widget(password, popup_chunks[1]);

	let service_text = Paragraph::new(app.input.service())
		.fg(Color::White)
		.block(service_block);
	frame.render_widget(service_text, popup_chunks[2]);

	let (chunk, text) = match app.input_mode {
		InputMode::Login => (popup_chunks[0], &app.input.login),
		InputMode::Password => (popup_chunks[1], &app.input.password),
		InputMode::Service => (popup_chunks[2], &app.input.service),
	};

	let width = chunk.width.max(3) - 3;
	let scroll = text.visual_scroll(width as usize);
	let x = text.visual_cursor().max(scroll) - scroll + 1;
	frame.set_cursor_position((chunk.x + x as u16, chunk.y + 1));
}

fn render_footer(app: &App, frame: &mut Frame, area: Rect) {
	let info_footer = Paragraph::new(Text::from_iter(INFO_TEXT))
		.style(Style::new().fg(app.colors.row_fg).bg(app.colors.buffer_bg))
		.centered()
		.block(
			Block::bordered()
				.border_type(BorderType::Double)
				.border_style(Style::new().fg(app.colors.footer_border_color)),
		);
	frame.render_widget(info_footer, area);
}

fn constraint_len_calculator<T: Data>(items: &[T]) -> (u16, u16, u16) {
	let name_len = items
		.iter()
		.map(Data::login)
		.map(UnicodeWidthStr::width)
		.max()
		.unwrap_or(0) as u16;

	let password_len: u16 = items
		.iter()
		.map(Data::password)
		.map(UnicodeWidthStr::width)
		.max()
		.unwrap_or(0) as u16;

	let service_len = items
		.iter()
		.map(Data::service)
		.map(UnicodeWidthStr::width)
		.max()
		.unwrap_or(0) as u16;

	(name_len, password_len, service_len)
}
