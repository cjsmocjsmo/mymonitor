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
use std::env;
use std::io;
use std::net::{IpAddr, UdpSocket};
use std::sync::mpsc::{Receiver, channel};
use std::thread;
use std::time::Duration;
use tokio::sync::broadcast;

mod alerting;
mod metrics;
mod stream;
mod ui {
    pub mod cpu_widget;
    pub mod dashboard;
    pub mod disk_widget;
    pub mod memory_widget;
    pub mod net_widget;
    pub mod theme;
}

const DEFAULT_WS_ADDR: &str = "0.0.0.0:9001";

enum RunMode {
    Ui,
    Stream,
}

struct Collectors {
    cpu: CpuCollector,
    mem: MemoryCollector,
    disk: DiskCollector,
    net: NetworkCollector,
}

impl Collectors {
    fn new() -> Self {
        Self {
            cpu: CpuCollector::new(),
            mem: MemoryCollector::new(),
            disk: DiskCollector::new(),
            net: NetworkCollector::new(),
        }
    }

    fn collect_snapshot(&mut self) -> MetricSnapshot {
        let (total_cpu_usage, core_cpu_usage) = self.cpu.collect();

        MetricSnapshot {
            timestamp: Local::now(),
            cpu_usage: total_cpu_usage,
            core_cpu_usage,
            total_memory: self.mem.collect_total(),
            used_memory: self.mem.collect_used(),
            disk_read: self.disk.collect_read(),
            disk_write: self.disk.collect_write(),
            net_rx: self.net.collect_rx(),
            net_tx: self.net.collect_tx(),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mode = parse_mode(env::args().skip(1))?;

    match mode {
        RunMode::Ui => run_ui_mode(),
        RunMode::Stream => run_stream_mode(),
    }
}

fn parse_mode<I>(args: I) -> Result<RunMode, Box<dyn std::error::Error>>
where
    I: Iterator<Item = String>,
{
    let mut mode = RunMode::Ui;

    for arg in args {
        match arg.as_str() {
            "-u" => mode = RunMode::Ui,
            "-s" => mode = RunMode::Stream,
            "-h" | "--help" => {
                println!("Usage: ./mymonitor [-u | -s]");
                println!("  -u   Launch terminal UI mode (default)");
                println!("  -s   Launch WebSocket JSON streaming mode");
                std::process::exit(0);
            }
            _ => {
                return Err(format!(
                    "Unknown argument '{}'. Use -u for UI mode or -s for stream mode.",
                    arg
                )
                .into());
            }
        }
    }

    Ok(mode)
}

fn run_ui_mode() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting monitor-rs in UI mode...");

    let mut collectors = Collectors::new();

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
        let snapshot = collectors.collect_snapshot();

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

fn run_stream_mode() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting monitor-rs in stream mode...");

    let ws_addr = env::var("MYMONITOR_WS_ADDR").unwrap_or_else(|_| DEFAULT_WS_ADDR.to_string());
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    runtime.block_on(async move {
        let (tx, _) = broadcast::channel::<MetricSnapshot>(32);

        let server_tx = tx.clone();
        let server_addr = ws_addr.clone();
        tokio::spawn(async move {
            if let Err(err) = stream::ws_server::run_server(&server_addr, server_tx).await {
                eprintln!("WebSocket server error: {}", err);
            }
        });

        let mut collectors = Collectors::new();
        loop {
            let snapshot = collectors.collect_snapshot();
            let _ = tx.send(snapshot);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    })
}

fn resolve_local_ip() -> Option<String> {
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    let addr = socket.local_addr().ok()?;

    match addr.ip() {
        IpAddr::V4(ip) => Some(ip.to_string()),
        IpAddr::V6(_) => None,
    }
}

fn launch_ui(ui_rx: Receiver<MetricSnapshot>) -> Result<(), io::Error> {
    let local_ip = resolve_local_ip().unwrap_or_else(|| "unknown-ip".to_string());

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
            terminal.draw(|f| ui::dashboard::render(f, &snapshot, &local_ip))?;
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
