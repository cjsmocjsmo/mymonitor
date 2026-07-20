use chrono::Local;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use alerting::handler::AlertEvaluator;
use metrics::snapshot::MetricSnapshot;
use metrics::{
    cpu::CpuCollector, disk::DiskCollector, memory::MemoryCollector, network::NetworkCollector,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::env;
use std::fs;
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
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

const WS_ADDR: &str = "0.0.0.0:9001";
const METRIC_INTERVAL_SECS: u64 = 2;

enum RunMode {
    Ui,
    Stream,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mode = match parse_mode(env::args().skip(1)) {
        Ok(mode) => mode,
        Err(message) => {
            eprintln!("{}", message);
            print_usage();
            return Err("invalid arguments".into());
        }
    };

    match mode {
        RunMode::Ui => run_ui_mode(),
        RunMode::Stream => run_stream_mode(),
    }
}

fn parse_mode<I>(mut args: I) -> Result<RunMode, String>
where
    I: Iterator<Item = String>,
{
    match args.next() {
        Some(flag) => {
            if args.next().is_some() {
                return Err("only one argument is supported".to_string());
            }

            match flag.as_str() {
                "-u" => Ok(RunMode::Ui),
                "-s" => Ok(RunMode::Stream),
                "-h" | "--help" => Err("".to_string()),
                _ => Err(format!("unknown argument: {}", flag)),
            }
        }
        None => Err("missing required mode argument".to_string()),
    }
}

fn print_usage() {
    eprintln!("Usage: mymonitor <-u|-s>");
    eprintln!("  -u    Run local UI only (no websocket streaming)");
    eprintln!("  -s    Run websocket streaming server only on ws://{}", WS_ADDR);
}

fn run_ui_mode() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting monitor-rs in UI mode...");

    // Initialize collectors
    let mut cpu = CpuCollector::new();
    let mut mem = MemoryCollector::new();
    let mut disk = DiskCollector::new();
    let mut net = NetworkCollector::new();
    let device_id = detect_device_id();
    let hostname = detect_hostname();

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
        let mut evaluator = AlertEvaluator::new();
        for snapshot in alert_rx {
            evaluator.evaluate_snapshot(&snapshot);
        }
    });

    // Metrics collection loop
    loop {
        let (cpu_usage, core_cpu_usage) = cpu.collect();
        let snapshot = MetricSnapshot {
            device_id: device_id.clone(),
            hostname: hostname.clone(),
            timestamp: Local::now(),
            cpu_usage,
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

        thread::sleep(Duration::from_secs(METRIC_INTERVAL_SECS));
    }

    Ok(())
}

fn run_stream_mode() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting monitor-rs in server mode on ws://{}...", WS_ADDR);

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    runtime.block_on(async move {
        let mut cpu = CpuCollector::new();
        let mut mem = MemoryCollector::new();
        let mut disk = DiskCollector::new();
        let mut net = NetworkCollector::new();
        let device_id = detect_device_id();
        let hostname = detect_hostname();

        let (tx, _) = broadcast::channel(64);
        let server_tx = tx.clone();

        tokio::spawn(async move {
            if let Err(err) = stream::ws_server::run_server(WS_ADDR, server_tx).await {
                eprintln!("WebSocket server error: {}", err);
            }
        });

        loop {
            let (total_cpu_usage, core_cpu_usage) = cpu.collect();

            let snapshot = MetricSnapshot {
                device_id: device_id.clone(),
                hostname: hostname.clone(),
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

            let _ = tx.send(snapshot);
            tokio::time::sleep(Duration::from_secs(METRIC_INTERVAL_SECS)).await;
        }
    })
}

fn detect_device_id() -> String {
    if let Ok(machine_id) = fs::read_to_string("/etc/machine-id") {
        let trimmed = machine_id.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    if let Ok(hostname) = std::env::var("HOSTNAME") {
        let trimmed = hostname.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    "unknown-device".to_string()
}

fn detect_hostname() -> String {
    if let Some(hostname) = get_system_hostname() {
        return hostname;
    }

    if let Ok(hostname) = fs::read_to_string("/etc/hostname") {
        let trimmed = hostname.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    if let Ok(hostname) = std::env::var("HOSTNAME") {
        let trimmed = hostname.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    "unknown-host".to_string()
}

fn get_system_hostname() -> Option<String> {
    let mut buffer = [0u8; 256];

    let result = unsafe { libc::gethostname(buffer.as_mut_ptr().cast(), buffer.len()) };
    if result != 0 {
        return None;
    }

    let hostname_len = buffer
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(buffer.len());

    let hostname = std::str::from_utf8(&buffer[..hostname_len]).ok()?.trim();
    if hostname.is_empty() {
        None
    } else {
        Some(hostname.to_string())
    }
}

fn launch_ui(ui_rx: Receiver<MetricSnapshot>) -> Result<(), io::Error> {
    let local_ip = detect_local_ip();

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

fn detect_local_ip() -> String {
    let bind_addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 0));
    let remote_addr = SocketAddr::from((Ipv4Addr::new(8, 8, 8, 8), 80));

    let socket = match UdpSocket::bind(bind_addr) {
        Ok(socket) => socket,
        Err(_) => return "unknown".to_string(),
    };

    if socket.connect(remote_addr).is_err() {
        return "unknown".to_string();
    }

    match socket.local_addr().map(|addr| addr.ip()) {
        Ok(IpAddr::V4(ip)) => ip.to_string(),
        Ok(IpAddr::V6(ip)) => ip.to_string(),
        Err(_) => "unknown".to_string(),
    }
}
