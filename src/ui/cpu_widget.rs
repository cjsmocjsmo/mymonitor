use crate::metrics::snapshot::MetricSnapshot;
use super::theme;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    prelude::*,
    widgets::{Block, Borders, Gauge, Paragraph},
};

pub fn render(f: &mut Frame, snapshot: &MetricSnapshot, area: Rect) {
    if area.height == 0 {
        return;
    }

    let panel_title = match snapshot.cpu_temp {
        Some(temp) => format!("CPU (Total + Cores) | Temp: {:.1}C", temp),
        None => "CPU (Total + Cores) | Temp: N/A".to_string(),
    };
    let panel = Block::default().title(panel_title).borders(Borders::ALL);
    let inner = panel.inner(area);
    f.render_widget(panel, area);

    if inner.height == 0 {
        return;
    }

    let visible_count = (snapshot.core_cpu_usage.len() + 1).min(inner.height as usize);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Length(1); visible_count])
        .split(inner);

    if let Some(total_row) = rows.first() {
        let total_usage = snapshot.cpu_usage;
        let ratio = (f64::from(total_usage) / 100.0).clamp(0.0, 1.0);
        let gauge = Gauge::default()
            .gauge_style(theme::default_style().fg(Color::Green))
            .ratio(ratio)
            .label(format!("{:>5.1}%", total_usage));
        f.render_widget(gauge, *total_row);
    }

    for (row, (idx, usage)) in rows
        .iter()
        .skip(1)
        .zip(snapshot.core_cpu_usage.iter().enumerate())
    {
        let core_line = Paragraph::new(format!("cpu{}: {:.0}%", idx + 1, usage));
        f.render_widget(core_line, *row);
    }
}
