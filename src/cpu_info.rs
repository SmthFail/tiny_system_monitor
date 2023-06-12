use sysinfo::{CpuExt, System, SystemExt};

pub struct CpuInfo {
    sys: System,
    cpus_usage: Vec<f64>,
    pub cpu_count: usize,
    ram_used: u64,
    ram_total: u64,
    swap_used: u64,
    swap_total: u64,
}

impl CpuInfo {
    pub fn new() -> Self {
        let mut sys = System::new();

        sys.refresh_cpu();

        let mut cpus_usage: Vec<f64> = Vec::new();
        for cpu in sys.cpus() {
            cpus_usage.push(cpu.cpu_usage() as f64 / 100.0);
        }

        sys.refresh_memory();
        let ram_used = sys.used_memory() / 1024 / 1024;
        let ram_total = sys.total_memory() / 1024 / 1024;

        let swap_used = sys.used_swap() / 1024 / 1024;
        let swap_total = sys.total_swap() / 1024 / 1024;

        CpuInfo {
            sys,
            cpu_count: cpus_usage.len(),
            cpus_usage,
            ram_used,
            ram_total,
            swap_used,
            swap_total,
        }
    }

    pub fn update(&mut self) {
        self.sys.refresh_cpu();
        for (ind, cpu) in self.sys.cpus().iter().enumerate() {
            self.cpus_usage[ind] = cpu.cpu_usage() as f64;
        }
    }

    pub fn get_cpu_usage(&self, cpu_index: usize) -> f64 {
        self.cpus_usage[cpu_index]
    }

    pub fn get_ram_usage(&self) -> (u64, u64) {
        (self.ram_used, self.ram_total)
    }

    pub fn get_swap_usage(&self) -> (u64, u64) {
        (self.swap_used, self.swap_total)
    }
}
