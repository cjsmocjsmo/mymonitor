use ratatui::style::{Color, Style};

pub fn default_style() -> Style {
    Style::default().fg(Color::White).bg(Color::Black)
}
