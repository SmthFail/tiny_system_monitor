use sysinfo::System;

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
        
        sys.refresh_cpu_all();
        let cpus_usage: Vec<f64> = vec![0.0; sys.cpus().len()];
        
        CpuInfo {
            sys,
            cpu_count: cpus_usage.len(),
            cpus_usage,
            ram_used: 0,
            ram_total: 1,
            swap_used: 0,
            swap_total: 1,
        }
    }

    pub fn update(&mut self) {  
        self.update_cpu_usage();
        self.update_memory_usage();
    }
    
    fn update_cpu_usage(&mut self) {
        self.sys.refresh_cpu_all();
        for (ind, cpu) in self.sys.cpus().iter().enumerate() {
            self.cpus_usage[ind] = cpu.cpu_usage() as f64;
        }
    }

    fn update_memory_usage(&mut self) {
        self.sys.refresh_memory();
        self.ram_used = self.sys.used_memory() / 1024 / 1024;
        self.ram_total = self.sys.total_memory() / 1024 / 1024;

        self.swap_used = self.sys.used_swap() / 1024 / 1024;
        self.swap_total = self.sys.total_swap() / 1024 / 1024;
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
