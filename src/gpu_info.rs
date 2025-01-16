use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::Device;


pub struct GpuDeviceInfo {
    pub gpu_info: String,
    pub memory_used: f64,
    pub memory_total: f64,
    pub utilization_rates: f64,
    pub temperature: u32,
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
