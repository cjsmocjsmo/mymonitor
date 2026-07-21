use crate::metrics::snapshot::MetricSnapshot;
use ratatui::{
    Frame,
    prelude::*,
    style::{Color, Style},
    widgets::{Block, Borders, Gauge},
};

pub fn render(f: &mut Frame, snapshot: &MetricSnapshot, area: Rect) {
    let usage = snapshot.disk_usage_pct.clamp(0.0, 100.0);

    let gauge = Gauge::default()
        .block(Block::default().title("Disk Usage").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Yellow))
        .percent(usage.round() as u16)
        .label(format!("{usage:.1}% used"));

    f.render_widget(gauge, area);
}
