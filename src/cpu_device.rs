use super::device_model::Device;
use super::app_config::DeviceTile;
use super::cpu_info::CpuInfo;
use super::ui::calculate_progress_bar;

pub fn create_device(device_tile: &DeviceTile) -> Box<dyn Device> {
    let mut cpu_device = Box::new(CpuDevice::new(device_tile));
    cpu_device.update();
    cpu_device
}

pub struct CpuDevice {
    name: String,
    row: u16, 
    col: u16,
    width: u16,
    height: u16,
    cpu_info: CpuInfo,
   pub  print_data: Vec<String>
}

impl Device for CpuDevice {
    fn new(device_tile: &DeviceTile) -> Self {
        let mut data_array: Vec<String> = vec!["_".repeat(device_tile.width.into()); device_tile.height.into()];
        let header: String = {
            let hs = "CPU_HEADER".to_string();
            let empty_space = device_tile.width as usize - hs.len();
            let line = hs.clone() +  &"|".repeat(empty_space);
            line
        };

        data_array[0] = header;

        CpuDevice{
            name: device_tile.name.clone(),
            width: device_tile.width,
            height: device_tile.height,
            row: device_tile.row,
            col: device_tile.col,
            cpu_info: CpuInfo::new(),
            print_data: data_array
        }
    }

    fn resize(&mut self) {
        // TODO print_data must be resized here
    }

    fn update(&mut self) {
        self.cpu_info.update();
    }    

    fn show(&mut self) -> &Vec<String> {
        for i in 0..self.cpu_info.cpu_count {
            let cpu_usage = self.cpu_info.get_cpu_usage(i); 
            let cpu_bar = calculate_progress_bar(
                self.width,
                format!("{:3}[", i),
                cpu_usage / 100.0,
                format!("{:.2}%]", cpu_usage),
            );
            self.print_data[i + 2] = cpu_bar;
        }
 
        &self.print_data
    }
}
