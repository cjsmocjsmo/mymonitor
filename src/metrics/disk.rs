use sysinfo::{Disks, System};

pub struct DiskCollector {
    sys: System,
    disks: Disks,
    prev_read: u64,
    prev_write: u64,
}

impl DiskCollector {
    pub fn new() -> Self {
        let mut sys = System::new_all(); // ensures process list is populated
        sys.refresh_all();
        let disks = Disks::new_with_refreshed_list();

        let (read, write) = Self::aggregate(&sys);
        DiskCollector {
            sys,
            disks,
            prev_read: read,
            prev_write: write,
        }
    }

    fn aggregate(sys: &System) -> (u64, u64) {
        let mut read = 0;
        let mut write = 0;
        for process in sys.processes().values() {
            let usage = process.disk_usage();
            read += usage.total_read_bytes;
            write += usage.total_written_bytes;
        }
        (read, write)
    }

    pub fn collect_read(&mut self) -> u64 {
        self.sys.refresh_all();
        let (read, _) = Self::aggregate(&self.sys);
        let delta = read.saturating_sub(self.prev_read);
        self.prev_read = read;
        delta
    }

    pub fn collect_write(&mut self) -> u64 {
        self.sys.processes();
        let (_, write) = Self::aggregate(&self.sys);
        let delta = write.saturating_sub(self.prev_write);
        self.prev_write = write;
        delta
    }

    pub fn collect_usage_pct(&mut self) -> f32 {
        self.disks.refresh(true);

        let mut total_space = 0u64;
        let mut used_space = 0u64;

        for disk in self.disks.list() {
            let total = disk.total_space();
            let available = disk.available_space();
            total_space = total_space.saturating_add(total);
            used_space = used_space.saturating_add(total.saturating_sub(available));
        }

        if total_space == 0 {
            return 0.0;
        }

        (used_space as f32 / total_space as f32) * 100.0
    }
}
