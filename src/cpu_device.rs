use super::device_model::Device;
use super::app_config::DeviceTile;

pub fn create_device(device_tile: &DeviceTile) -> Box<dyn Device> {
    Box::new(CpuDevice::new(device_tile))
}

struct CpuDevice {
    name: String,
    row: u16, 
    col: u16,
    width: u16,
    height: u16
}

impl Device for CpuDevice {
    fn new(device_tile: &DeviceTile) -> Self {
        CpuDevice{
            name: device_tile.name.clone(),
            width: device_tile.width,
            height: device_tile.height,
            row: device_tile.row,
            col: device_tile.col
        }
    }

    fn update(&mut self) {

    }    

    fn show(&self) {
        println!("Cpu device: {} {}", self.width, self.height);
    }
}
