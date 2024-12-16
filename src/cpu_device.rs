use super::device_model::Device;
use super::app_config::DeviceTile;
use super::cpu_info::CpuInfo;

pub fn create_device(device_tile: &DeviceTile) -> Box<dyn Device> {
    Box::new(CpuDevice::new(device_tile))
}

struct CpuDevice {
    name: String,
    row: u16, 
    col: u16,
    width: u16,
    height: u16,
    cpu_info: CpuInfo,
}

impl Device for CpuDevice {
    fn new(device_tile: &DeviceTile) -> Self {
        CpuDevice{
            name: device_tile.name.clone(),
            width: device_tile.width,
            height: device_tile.height,
            row: device_tile.row,
            col: device_tile.col,
            cpu_info: CpuInfo::new()
        }
    }

    fn update(&mut self) {
        self.cpu_info.update();
    }    

    fn show(&self) {
        println!("Cpu device: {} {}", self.width, self.height);
    }
}
