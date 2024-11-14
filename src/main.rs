use::std::env;
mod file_config;
use crate::file_config::{FileConfig, FileDevice};


#[derive(Debug)]
struct DeviceTile {
    name: String,
   row: u8,
   col: u8,
   width: u8,
   height: u8
}


#[derive(Debug)]
struct AppConfig {
   name: String,
   devices: Vec<FileDevice>, 
   tiles: Vec<DeviceTile> 
}


impl AppConfig {
    pub fn new(config_name: String) -> Self {
        let file_config = FileConfig::new(config_name);
        let device_tiles = Self::get_device_tiles(&file_config.devices);
        AppConfig{
             name: file_config.name,
             devices: file_config.devices,
             tiles: device_tiles
        }
    }

    fn get_device_tiles(devices: &Vec<FileDevice>) -> Vec<DeviceTile> {
        let mut tiles: Vec<DeviceTile> = Vec::new();
        let scale: (f32, f32) = Self::get_tile_scale(&devices.clone(), 100, 100);
        for device in devices {
            tiles.push(
                DeviceTile {
                    name: device.device_type.clone(),
                    row: (device.top_left.0 as f32 * scale.0) as u8,
                    col: (device.top_left.1 as f32 * scale.1) as u8,
                    width: (device.width as f32 * scale.0) as u8,
                    height: (device.height as f32 * scale.1) as u8
                }
            )
        }
        tiles
    }

    fn get_tile_scale(devices: &Vec<FileDevice>, term_width: u8, term_height:u8) -> (f32, f32)  {
        // get grid
       let mut max_w = 1;
       let mut max_h = 1;
       for device in devices {
           let device_max_w = device.top_left.1 + device.width;
           if device_max_w > max_w {
               max_w = device_max_w;
           }
           let device_max_h = device.top_left.0 + device.height;
           if device_max_h > max_h {
               max_h = device_max_h;
           }
       }
       let col_scale: f32 = (term_width / max_w).into();
       let row_scale: f32 = (term_height / max_h).into();


       println!("Max w {} and max h {}", max_w, max_h);
       println!("Row scale {} and col scale {}", row_scale, col_scale);
       (row_scale, col_scale)
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
  
    let config = match args.len() - 1 {
        0 => AppConfig::new(String::new()),
        1 => AppConfig::new(args[1].clone()),
        _ => { 
            eprint!("ERROR: Invalid number of arguments");
            return;
        }
    };
    println!("Config: {} loaded!", config.name);
    println!("{:?}", &config)
}
