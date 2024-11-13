use::std::env;
mod file_config;
use crate::file_config::{FileConfig, FileDevice};


#[derive(Debug)]
struct AppConfig {
    name: String,
    devices: Vec<FileDevice> 
    
}


impl AppConfig {
    pub fn new(config_name: String) -> Self {
        let file_config = FileConfig::new(config_name);
        let config = AppConfig{
             name: file_config.name,
             devices: file_config.devices,
        };
        config.validate_devices();
        config
    }

    fn validate_devices(&self)  {
        // get grid
       let mut max_w = 1;
       let mut max_h = 1;
       for device in &self.devices {
           let device_max_w = device.top_left.1 + device.width;
           if device_max_w > max_w {
               max_w = device_max_w;
           }
           let device_max_h = device.top_left.0 + device.height;
           if device_max_h > max_h {
               max_h = device_max_h;
           }
       } 
       println!("Max w {} and max h {}", max_w, max_h);
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
