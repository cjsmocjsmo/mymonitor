use chrono::{DateTime, Local};
extern crate serde;
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct MetricSnapshot {
    pub device_id: String,
    pub hostname: String,
    pub timestamp: DateTime<Local>,
    pub cpu_usage: f32,
    pub core_cpu_usage: Vec<f32>,
    pub total_memory: u64,
    pub used_memory: u64,
    pub disk_read: u64,
    pub disk_write: u64,
    pub disk_usage_pct: f32,
    pub net_rx: u64,
    pub net_tx: u64,
}
