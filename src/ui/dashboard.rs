use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    widgets::Paragraph,
};

use super::{cpu_widget, disk_widget, memory_widget, net_widget};
use crate::metrics::snapshot::MetricSnapshot;

pub fn render(f: &mut Frame, snapshot: &MetricSnapshot, hostname: &str) {
    let cpu_rows = (snapshot.core_cpu_usage.len() as u16).saturating_add(3);
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(cpu_rows),
            Constraint::Length(3),
            Constraint::Length(4),
            Constraint::Length(4),
            Constraint::Min(0),
        ])
        .split(f.area());

    let host_line = Paragraph::new(format!("Host: {hostname}"));
    f.render_widget(host_line, layout[0]);

    cpu_widget::render(f, snapshot, layout[1]);
    memory_widget::render(f, snapshot, layout[2]);
    disk_widget::render(f, snapshot, layout[3]);
    net_widget::render(f, snapshot, layout[4]);
}
