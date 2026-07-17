use std::env;

const DEFAULT_CPU_HIGH_THRESHOLD: f64 = 85.0;
const DEFAULT_CPU_RECOVER_THRESHOLD: f64 = 75.0;
const DEFAULT_CPU_SUSTAINED_SAMPLES: u32 = 10;
const DEFAULT_MEMORY_HIGH_THRESHOLD: f64 = 70.0;

pub struct AlertConfig {
    pub cpu_high_threshold: f64,
    pub cpu_recover_threshold: f64,
    pub cpu_sustained_samples: u32,
    pub memory_high_threshold: f64,
}

pub fn load_alert_config() -> AlertConfig {
    let cpu_high_threshold = env_or_default_f64("MYMONITOR_CPU_HIGH_THRESHOLD", DEFAULT_CPU_HIGH_THRESHOLD);
    let cpu_recover_threshold =
        env_or_default_f64("MYMONITOR_CPU_RECOVER_THRESHOLD", DEFAULT_CPU_RECOVER_THRESHOLD);
    let cpu_sustained_samples =
        env_or_default_u32("MYMONITOR_CPU_SUSTAINED_SAMPLES", DEFAULT_CPU_SUSTAINED_SAMPLES);
    let memory_high_threshold =
        env_or_default_f64("MYMONITOR_MEMORY_HIGH_THRESHOLD", DEFAULT_MEMORY_HIGH_THRESHOLD);

    validate_percentage("MYMONITOR_CPU_HIGH_THRESHOLD", cpu_high_threshold);
    validate_percentage("MYMONITOR_CPU_RECOVER_THRESHOLD", cpu_recover_threshold);
    validate_percentage("MYMONITOR_MEMORY_HIGH_THRESHOLD", memory_high_threshold);

    if cpu_recover_threshold >= cpu_high_threshold {
        panic!(
            "Invalid alert config: MYMONITOR_CPU_RECOVER_THRESHOLD ({}) must be lower than MYMONITOR_CPU_HIGH_THRESHOLD ({})",
            cpu_recover_threshold,
            cpu_high_threshold
        );
    }

    if cpu_sustained_samples == 0 {
        panic!(
            "Invalid alert config: MYMONITOR_CPU_SUSTAINED_SAMPLES must be >= 1, got {}",
            cpu_sustained_samples
        );
    }

    AlertConfig {
        cpu_high_threshold,
        cpu_recover_threshold,
        cpu_sustained_samples,
        memory_high_threshold,
    }
}

fn env_or_default_f64(name: &str, default: f64) -> f64 {
    match env::var(name) {
        Ok(raw) => raw.parse::<f64>().unwrap_or_else(|_| {
            panic!(
                "Invalid value for {}: '{}' is not a valid number",
                name,
                raw
            )
        }),
        Err(_) => default,
    }
}

fn env_or_default_u32(name: &str, default: u32) -> u32 {
    match env::var(name) {
        Ok(raw) => raw.parse::<u32>().unwrap_or_else(|_| {
            panic!(
                "Invalid value for {}: '{}' is not a valid unsigned integer",
                name,
                raw
            )
        }),
        Err(_) => default,
    }
}

fn validate_percentage(name: &str, value: f64) {
    if !(0.0..=100.0).contains(&value) {
        panic!(
            "Invalid value for {}: {} is outside allowed range [0, 100]",
            name,
            value
        );
    }
}
