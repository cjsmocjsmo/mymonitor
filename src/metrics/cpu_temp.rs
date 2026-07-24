use std::fs;
use std::path::PathBuf;

const THERMAL_SYSFS_DIR: &str = "/sys/class/thermal";

pub struct CpuTempCollector {
    thermal_zone_paths: Vec<PathBuf>,
}

impl CpuTempCollector {
    pub fn new() -> Self {
        Self {
            thermal_zone_paths: discover_thermal_zone_paths(),
        }
    }

    pub fn collect(&mut self) -> Option<f32> {
        if self.thermal_zone_paths.is_empty() {
            self.thermal_zone_paths = discover_thermal_zone_paths();
        }

        let mut values = Vec::new();

        for temp_path in &self.thermal_zone_paths {
            if let Some(celsius) = read_temp_celsius(temp_path) {
                values.push(celsius);
            }
        }

        if values.is_empty() {
            return None;
        }

        let sum: f32 = values.iter().sum();
        Some(sum / values.len() as f32)
    }
}

fn discover_thermal_zone_paths() -> Vec<PathBuf> {
    let mut preferred = Vec::new();
    let mut fallback = Vec::new();

    let entries = match fs::read_dir(THERMAL_SYSFS_DIR) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    for entry in entries.flatten() {
        let zone_path = entry.path();

        let Some(zone_name) = zone_path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if !zone_name.starts_with("thermal_zone") {
            continue;
        }

        let temp_path = zone_path.join("temp");
        if !temp_path.exists() {
            continue;
        }

        fallback.push(temp_path.clone());

        let zone_type_path = zone_path.join("type");
        let zone_type = fs::read_to_string(zone_type_path)
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase();

        // Prefer sensors likely tied to CPU/package temperatures.
        if zone_type.contains("cpu") || zone_type.contains("package") || zone_type.contains("soc") {
            preferred.push(temp_path);
        }
    }

    if preferred.is_empty() {
        fallback
    } else {
        preferred
    }
}

fn read_temp_celsius(temp_path: &PathBuf) -> Option<f32> {
    let raw = fs::read_to_string(temp_path).ok()?;
    let value = raw.trim().parse::<f32>().ok()?;

    if value > 1000.0 {
        Some(value / 1000.0)
    } else {
        Some(value)
    }
}