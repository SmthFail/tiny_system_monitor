use crate::file_config::{FileConfig, FileDevice};


#[derive(Debug)]
pub struct DeviceTile {
   pub name: String,
   pub row: u16,
   pub col: u16,
   pub width: u16,
   pub height: u16
}

#[derive(Debug)]
pub struct AppConfig {
   pub name: String,
   pub symbol: String,
   pub devices: Vec<FileDevice>, 
   pub tiles: Vec<DeviceTile> 
}


impl AppConfig {
    pub fn new(config_name: String, screen_width: u16, screen_height: u16) -> Self {
        let file_config = FileConfig::new(config_name);
        let device_tiles = Self::get_device_tiles(&file_config.devices, screen_width, screen_height);
        AppConfig{
             name: file_config.name,
             symbol: file_config.symbol,
             devices: file_config.devices,
             tiles: device_tiles
        }
    }

    pub fn update_grid(&mut self, new_w: u16, new_h: u16) {
        self.tiles = Self::get_device_tiles(&self.devices, new_w, new_h);
    }

    fn get_device_tiles(devices: &Vec<FileDevice>, new_w: u16, new_h: u16) -> Vec<DeviceTile> {
        let mut tiles: Vec<DeviceTile> = Vec::new();
        let (col_scale, row_scale) = Self::get_tile_scale(&devices.clone(), new_w, new_h);
        for device in devices {
            tiles.push(
                DeviceTile {
                    name: device.device_type.clone(),
                    row: (device.top_left.0 as f32 * row_scale) as u16,
                    col: (device.top_left.1 as f32 * col_scale) as u16,
                    width: (device.width as f32 * col_scale) as u16,
                    height: (device.height as f32 * row_scale) as u16
                }
            )
        }
        tiles
    }

    fn get_tile_scale(devices: &Vec<FileDevice>, term_width: u16, term_height:u16) -> (f32, f32)  {
        // get grid
       let mut max_w= 1;
       let mut max_h= 1;
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

       (col_scale, row_scale)
    }
}
