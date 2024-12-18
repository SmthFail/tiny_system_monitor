use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::{Device, Nvml};

pub struct GpuAll {
    nvml: Nvml,
    gpu_devices: Vec<GpuDeviceInfo>,
    pub device_count: u32,
}

impl GpuAll {
    pub fn new() -> Self {
        let nvml = Nvml::init().unwrap();
        let device_count = nvml.device_count().unwrap();
        let mut gpu_devices: Vec<GpuDeviceInfo> = Vec::new();

        for i in 0..device_count {
            let device = nvml.device_by_index(i).unwrap();
            gpu_devices.push(GpuDeviceInfo::new(device))
        }
        GpuAll {
            nvml,
            gpu_devices,
            device_count,
        }
    }
    pub fn get_info(&self, device_index: u32) -> String {
        format!(
            "{}, T: {:>3}°C",
            self.gpu_devices[device_index as usize].gpu_info,
            self.gpu_devices[device_index as usize].temperature,
        )
    }

    pub fn get_memory_info(&self, device_index: u32) -> (f64, f64) {
        (
            self.gpu_devices[device_index as usize].memory_used,
            self.gpu_devices[device_index as usize].memory_total,
        )
    }

    pub fn get_utilization_rate_info(&self, device_index: u32) -> f64 {
        self.gpu_devices[device_index as usize].utilization_rates
    }

    pub fn update(&mut self) {
        for ind in 0..self.device_count {
            let device = self.nvml.device_by_index(ind).unwrap();
            let _ = &mut self.gpu_devices[ind as usize].update(device);
        }
    }
}

pub struct GpuDeviceInfo {
    gpu_info: String,
    memory_used: f64,
    memory_total: f64,
    utilization_rates: f64,
    temperature: u32,
}

impl GpuDeviceInfo {
    pub fn new(device: Device) -> Self {
        let name: String = device.name().expect("Can't read GPU device name");
        let capability = match device.cuda_compute_capability() {
            Ok(compute_capability) => {
                format!("{}.{}", compute_capability.major, compute_capability.minor)
            }
            Err(_err) => "".to_string(),
        };

        let info = format!("{}, Cap: {}", name, capability);

        GpuDeviceInfo {
            gpu_info: info,
            memory_total: 1.0,
            memory_used: 0.0,
            utilization_rates: 0.0,
            temperature: 0,
        }
    }

    pub fn update(&mut self, device: Device) {
        self.temperature = match device.temperature(TemperatureSensor::Gpu) {
            Ok(temperature) => temperature,
            Err(_err) => panic!("Can't read temperature"),
        };

        let memory_info = match device.memory_info() {
            Ok(memory_info) => memory_info,
            Err(_err) => panic!("{}", _err),
        };

        // convert memory data to mb
        self.memory_used = (memory_info.used / 1024 / 1024) as f64;
        self.memory_total = (memory_info.total / 1024 / 1024) as f64;

        self.utilization_rates = match device.utilization_rates() {
            Ok(utilization_rates) => utilization_rates.gpu as f64,
            Err(_err) => panic!("{}", _err),
        };
    }
}
