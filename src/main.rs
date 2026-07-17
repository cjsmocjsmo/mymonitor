use chrono::Local;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use metrics::snapshot::MetricSnapshot;
use metrics::{
    cpu::CpuCollector, disk::DiskCollector, memory::MemoryCollector, network::NetworkCollector,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;
use std::sync::mpsc::{Receiver, channel};
use std::thread;
use std::time::Duration;

mod alerting;
mod metrics;
mod ui {
    pub mod cpu_widget;
    pub mod dashboard;
    pub mod disk_widget;
    pub mod memory_widget;
    pub mod net_widget;
    pub mod theme;
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting monitor-rs...");

    // Initialize collectors
    let mut cpu = CpuCollector::new();
    let mut mem = MemoryCollector::new();
    let mut disk = DiskCollector::new();
    let mut net = NetworkCollector::new();

    let (ui_tx, ui_rx) = channel();
    let (alert_tx, alert_rx) = channel();

    // Spawn the UI thread
    thread::spawn(move || {
        if let Err(e) = launch_ui(ui_rx) {
            eprintln!("Error in UI thread: {}", e);
        }
    });

    // Spawn the alert handling thread
    thread::spawn(move || {
        let mut alert_evaluator = alerting::handler::AlertEvaluator::new();
        for snapshot in alert_rx {
            alert_evaluator.evaluate_snapshot(&snapshot);
        }
    });

    // Metrics collection loop
    loop {
        let (total_cpu_usage, core_cpu_usage) = cpu.collect();

        let snapshot = MetricSnapshot {
            timestamp: Local::now(),
            cpu_usage: total_cpu_usage,
            core_cpu_usage,
            total_memory: mem.collect_total(),
            used_memory: mem.collect_used(),
            disk_read: disk.collect_read(),
            disk_write: disk.collect_write(),
            net_rx: net.collect_rx(),
            net_tx: net.collect_tx(),
        };

        if ui_tx.send(snapshot.clone()).is_err() {
            eprintln!("UI thread disconnected; shutting down monitor loop.");
            break;
        }

        if alert_tx.send(snapshot).is_err() {
            eprintln!("Alert thread disconnected; shutting down monitor loop.");
            break;
        }

        thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}

fn launch_ui(ui_rx: Receiver<MetricSnapshot>) -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }

        if let Ok(snapshot) = ui_rx.try_recv() {
            terminal.draw(|f| ui::dashboard::render(f, &snapshot))?;
        }
    }

    disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
