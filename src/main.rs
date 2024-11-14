use::std::env;
mod file_config;
use crate::file_config::{FileConfig, FileDevice};

use std::io::{stdout, Write};

use crossterm::event::{poll, read, Event, KeyEvent, KeyCode, KeyModifiers};
use crossterm::{execute, cursor};
use crossterm::terminal::{
    self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen
};

use std::time::Duration;

mod cpu_info;
mod gpu_info;
mod ui;

use crate::ui::LayoutBbox;
use ui::{LayoutType, Ui};




#[derive(Debug)]
struct DeviceTile {
    name: String,
   row: u16,
   col: u16,
   width: u16,
   height: u16
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
        let device_tiles = Self::get_device_tiles(&file_config.devices, 100, 100);
        AppConfig{
             name: file_config.name,
             devices: file_config.devices,
             tiles: device_tiles
        }
    }

    fn update_grid(&mut self, new_w: u16, new_h: u16) {
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


       println!("Max w {} and max h {}", max_w, max_h);
       println!("Row scale {} and col scale {}", row_scale, col_scale);
       (col_scale, row_scale)
    }
}

fn main() {
    //read and generate app config
    let args: Vec<String> = env::args().collect();
  
    let mut config = match args.len() - 1 {
        0 => AppConfig::new(String::new()),
        1 => AppConfig::new(args[1].clone()),
        _ => { 
            eprint!("ERROR: Invalid number of arguments");
            return;
        }
    };
    println!("Config: {} loaded!", config.name);
    println!("{:?}", &config);

    // start main loop    
    let mut stdout = stdout();
    
    let (cols, rows) = terminal::size().expect(
        "Can't get terminal size"
    );

    config.update_grid(cols, rows);


    let mut ui = Ui::new(cols, rows);
    println!("Terminal size: {} {}", cols, rows);
    ui.create_layout(
        config.tiles[0].name.clone(),
        LayoutBbox {
            top: config.tiles[0].row,
            left: config.tiles[0].col,
            width: config.tiles[0].width,
            height: config.tiles[0].height,
        },
        LayoutType::Cpu,
    );
    ui.create_layout(
        config.tiles[1].name.clone(),
        LayoutBbox {
            top: config.tiles[1].row,
            left: config.tiles[1].col,
            width: config.tiles[1].width,
            height: config.tiles[1].height,
        },
        LayoutType::Gpu,
    );

    execute!(stdout, EnterAlternateScreen, cursor::Hide,).unwrap();

    enable_raw_mode().unwrap();


    loop {
        ui.update_all(&config.tiles);

        stdout.flush().unwrap();
        if poll(Duration::from_millis(500)).unwrap() {
            match read().unwrap() {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => {
                    execute!(stdout, LeaveAlternateScreen, cursor::Show).unwrap();
                    break;
                }
                Event::Resize(width, height) => {
                    println!("Terminal resized");
                    ui.width = width;
                    ui.height = height;
                    config.update_grid(width, height);
                },
                _ => (),
            }
        }
        stdout.flush().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(500));
        
    }
    disable_raw_mode().unwrap();

}
