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
    symbol: String,
   pub  print_data: Vec<String>,
    padding: u16
}

impl Device for CpuDevice {
    fn new(device_tile: &DeviceTile) -> Self {
        let mut data_array: Vec<String> = vec!["".repeat(device_tile.width.into()); device_tile.height.into()];
        let header = "CPU:".to_string();
        data_array[0] = header;
        let padding = 1;
        CpuDevice{
            name: device_tile.name.clone(),
            width: device_tile.width - 2 * padding,
            height: device_tile.height - 2 * padding,
            row: device_tile.row + padding,
            col: device_tile.col + padding,
            cpu_info: CpuInfo::new(),
            print_data: data_array,
            symbol: device_tile.symbol.clone(),
            padding: 1
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
        self.cpu_info.update();
    }    

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn show(&mut self) -> &Vec<String> {
        for i in 0..self.cpu_info.cpu_count {
            let cpu_usage = self.cpu_info.get_cpu_usage(i); 
            let cpu_bar = calculate_progress_bar(
                self.width,
                format!("{:3}[", i).as_str(),
                cpu_usage / 100.0,
                format!("{:>5.1}%]", cpu_usage).as_str(),
                &self.symbol
            );
            self.print_data[i + 1] = cpu_bar;
        }

        let ram_usage = self.cpu_info.get_ram_usage();
        self.print_data[self.cpu_info.cpu_count + 2] = calculate_progress_bar(
            self.width,
            "RAM[",
            ram_usage.0 as f64 / ram_usage.1 as f64, 
            format!("{}/{}Mb]", ram_usage.0, ram_usage.1).as_str(),
            &self.symbol
        );         

        let swap_usage = self.cpu_info.get_swap_usage();
        self.print_data[self.cpu_info.cpu_count + 3] = calculate_progress_bar(
            self.width, 
            "SWP[",
            swap_usage.0 as f64 / swap_usage.1 as f64, 
            format!("{}/{}Mb]", swap_usage.0, swap_usage.1).as_str(),
            &self.symbol
        );
 
        &self.print_data
    }
}
