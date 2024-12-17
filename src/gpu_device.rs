use super::device_model::Device;
use super::app_config::DeviceTile;

pub fn create_device(device_tile: &DeviceTile) -> Box<dyn Device> {
    Box::new(GpuDevice::new(device_tile))
}

struct GpuDevice {
    name: String,
    col: u16,
    row: u16,
    width: u16,
    height: u16,
    print_data: Vec<String>
}

impl Device for GpuDevice {
    fn new(device_tile: &DeviceTile) -> Self {
        GpuDevice {
            name: device_tile.name.clone(),
            width: device_tile.width,
            height: device_tile.height,
            col: device_tile.col,
            row: device_tile.row,
            print_data: vec!["0".to_string(); device_tile.width.into()]
        }
    }
    fn resize(&mut self) {

    }

    fn update(&mut self) {

    }    

    fn show(&mut self) -> &Vec<String> {
        &self.print_data
    }
}
