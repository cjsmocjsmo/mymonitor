use sysinfo::System;
pub struct CpuCollector {
    sys: System,
}

impl CpuCollector {
    pub fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_cpu_all();
        CpuCollector { sys }
    }

    pub fn collect(&mut self) -> (f32, Vec<f32>) {
        self.sys.refresh_cpu_all();
        let total = self.sys.global_cpu_usage();
        let per_core = self.sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
        (total, per_core)
    }
}
