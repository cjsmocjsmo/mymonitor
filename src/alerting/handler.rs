use crate::alerting::rules::{AlertConfig, load_alert_config};
use crate::metrics::snapshot::MetricSnapshot;
use std::fs::OpenOptions;
use std::io::Write;

pub struct AlertEvaluator {
    config: AlertConfig,
    cpu_state: CpuAlertState,
    memory_alert_active: bool,
}

#[derive(Default)]
struct CpuAlertState {
    active: bool,
    consecutive_high_samples: u32,
}

impl AlertEvaluator {
    pub fn new() -> Self {
        Self {
            config: load_alert_config(),
            cpu_state: CpuAlertState::default(),
            memory_alert_active: false,
        }
    }

    pub fn evaluate_snapshot(&mut self, snapshot: &MetricSnapshot) {
        self.evaluate_cpu(snapshot);
        self.evaluate_memory(snapshot);
    }

    fn evaluate_cpu(&mut self, snapshot: &MetricSnapshot) {
        let cpu_usage = snapshot.cpu_usage as f64;

        if self.cpu_state.active {
            if cpu_usage <= self.config.cpu_recover_threshold {
                self.cpu_state.active = false;
                self.cpu_state.consecutive_high_samples = 0;
                self.write_log(format!(
                    "[RECOVERY] High CPU Usage recovered at {}. Current: {:.2}. Recover Threshold: {:.2}\n",
                    snapshot.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    cpu_usage,
                    self.config.cpu_recover_threshold
                ));
            }

            return;
        }

        if cpu_usage > self.config.cpu_high_threshold {
            self.cpu_state.consecutive_high_samples += 1;

            if self.cpu_state.consecutive_high_samples >= self.config.cpu_sustained_samples {
                self.cpu_state.active = true;
                self.write_log(format!(
                    "[ALERT] High CPU Usage triggered at {}. Threshold: {:.2}. Current: {:.2}. Sustained Samples: {}\n",
                    snapshot.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    self.config.cpu_high_threshold,
                    cpu_usage,
                    self.config.cpu_sustained_samples
                ));
            }

            return;
        }

        self.cpu_state.consecutive_high_samples = 0;
    }

    fn evaluate_memory(&mut self, snapshot: &MetricSnapshot) {
        if snapshot.total_memory == 0 {
            return;
        }

        let memory_usage_pct = (snapshot.used_memory as f64 / snapshot.total_memory as f64) * 100.0;

        if self.memory_alert_active {
            if memory_usage_pct <= self.config.memory_high_threshold {
                self.memory_alert_active = false;
                self.write_log(format!(
                    "[RECOVERY] High Memory Usage recovered at {}. Current: {:.2}. Recover Threshold: {:.2}\n",
                    snapshot.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    memory_usage_pct,
                    self.config.memory_high_threshold
                ));
            }

            return;
        }

        if memory_usage_pct > self.config.memory_high_threshold {
            self.memory_alert_active = true;
            self.write_log(format!(
                "[ALERT] High Memory Usage triggered at {}. Threshold: {:.2}. Current: {:.2}\n",
                snapshot.timestamp.format("%Y-%m-%d %H:%M:%S"),
                self.config.memory_high_threshold,
                memory_usage_pct
            ));
        }
    }

    fn write_log(&self, log_entry: String) {
        let mut log_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("alerts.log")
            .expect("Unable to open alert log file");

        log_file
            .write_all(log_entry.as_bytes())
            .expect("Failed to write to alert log file");
    }
}
