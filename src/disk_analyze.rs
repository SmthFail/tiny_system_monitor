use std::any::Any;

use crate::ui::calculate_progress_bar;

use super::device_model::Device;
use super::app_config::DeviceTile;
use sysinfo::{Disks, System};

pub fn create_device(device_tile: &DeviceTile) -> Box<dyn Device> {
    let mut cpu_device = Box::new(DiskDevice::new(device_tile));
    cpu_device.update();
    cpu_device
}

pub struct DiskDevice {
    name: String,
    row: u16, 
    col: u16,
    width: u16,
    height: u16,
   pub  print_data: Vec<String>,
    padding: u16,
    disks: Disks,
    symbol: String    
}

impl Device for DiskDevice {
    fn new(device_tile: &DeviceTile) -> Self {
        let mut data_array: Vec<String> = vec!["".repeat(device_tile.width.into()); device_tile.height.into()];
        let header = "Disks:".to_string();
        data_array[0] = header;
        let padding = 1;

        let mut sys = System::new_all();
        sys.refresh_all();
        let disks = Disks::new_with_refreshed_list();

        DiskDevice{
            name: device_tile.name.clone(),
            width: device_tile.width - 2 * padding,
            height: device_tile.height - 2 * padding,
            row: device_tile.row + padding,
            col: device_tile.col + padding,
            print_data: data_array,
            padding: 1,
            symbol: device_tile.symbol.clone(),
            disks
        }
    }

    fn resize(&mut self, tile: &DeviceTile) {
        self.width = tile.width - 2 * self.padding;
        self.height = tile.height - 2 * self.padding;
        self.col = tile.col + self.padding;
        self.row = tile.row + self.padding;
    }

    fn get_position(&self) -> (u16, u16) {
        (self.row, self.col)
    }

    fn get_size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn update(&mut self) {
        self.disks.refresh(true);
    }    

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn show(&mut self) -> &Vec<String> {
         for (i, disk) in self.disks.iter().enumerate() {
            let cur_pos = i + i * 2;
            self.print_data[cur_pos + 1] = format!(
                "{:?} | {}", 
                disk.name(), 
                disk.kind()
            );
            let disk_usage = (
                disk.total_space() - disk.available_space()) as f64 / disk.total_space() as f64;
            self.print_data[cur_pos + 2] = calculate_progress_bar(
                self.width, 
                "Usage: [", 
                disk_usage, 
                format!("{}/{}]GB", 
                    (disk.total_space() - disk.available_space()) / 1000000000, 
                    disk.total_space() / 1000000000
                    ).as_str(), 
                &self.symbol);
         }       
         &self.print_data
    }
}
