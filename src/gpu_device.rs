use super::device_model::Device;
use super::app_config::DeviceTile;
use nvml_wrapper::{Nvml};
use super::gpu_info::GpuDeviceInfo;
use crate::ui::calculate_progress_bar;

pub fn create_device(device_tile: &DeviceTile) -> Box<dyn Device> {
    Box::new(GpuDevice::new(device_tile))
}

struct GpuDevice {
    name: String,
    col: u16,
    row: u16,
    width: u16,
    height: u16,
    print_data: Vec<String>,
    nvml: Nvml,
    devices: Vec<GpuDeviceInfo>
}

impl Device for GpuDevice {
    fn new(device_tile: &DeviceTile) -> Self {
        let mut print_data: Vec<String> = vec!["".to_string(); device_tile.width.into()];
        
        print_data[0] = "GPU".to_string();

        let nvml = Nvml::init().unwrap();
        let device_count = nvml.device_count().unwrap();


        let mut devices: Vec<GpuDeviceInfo> = Vec::new();
        for i in 0..device_count {
            let device = nvml.device_by_index(i).unwrap();
            devices.push(GpuDeviceInfo::new(device))
        }

        GpuDevice {
            name: device_tile.name.clone(),
            width: device_tile.width,
            height: device_tile.height,
            col: device_tile.col,
            row: device_tile.row,
            print_data,
            nvml,
            devices
        }
    }
    fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn get_position(&self) -> (u16, u16) {
        (self.row, self.col)
    }

    fn update(&mut self) {
        for i in 0..self.devices.len() {
            let device = self.nvml.device_by_index(i as u32).unwrap();
            self.devices[i].update(device);            
        }
    }    

    fn show(&mut self) -> &Vec<String> {
        for (i, device) in self.devices.iter().enumerate() {
            let curr_pos = i + i * 3;
            self.print_data[curr_pos + 1] = format!(
                "{}, T: {:>3}°C", 
                device.gpu_info,
                device.temperature
            );
            self.print_data[curr_pos + 2] = calculate_progress_bar(
                self.width,
                String::from("MEM["),
                device.memory_used / device.memory_total,
                format!("{}/{}MB]", device.memory_used, device.memory_total)                
            );
            self.print_data[curr_pos + 3] = calculate_progress_bar(
                self.width, 
                String::from("GPU["), 
                device.utilization_rates / 100.0, 
                format!("{}%]", device.utilization_rates)
            )
        }
        &self.print_data
    }
}
