use::std::env;
use::std::process::exit;
mod app_config;
mod file_config;
use crate::app_config::{AppConfig, DeviceTile};

use std::io::{stdout, Write};

use crossterm::event::{poll, read, Event, KeyEvent, KeyCode, KeyModifiers};
use crossterm::{execute, cursor};
use crossterm::cursor::MoveTo;
use crossterm::style::Print;
use crossterm::terminal::{
    self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen
};

use std::time::Duration;

mod cpu_info;
mod gpu_info;
mod ui;

use crate::ui::LayoutBbox;
use ui::{LayoutType, Ui};

mod device_model;
use device_model::{Device, DEVICE_REGISTRY};
mod cpu_device;
mod gpu_device;

fn print_usage_message() {
    println!("Usage: ");
    println!("tsm  [Options]");
    println!("Options:");
    println!("  <config_name>   Read config with the given name. Config must be placed in ~/.config/tsm/<config_name>.json");
    println!("  -h, --help      Print help message")
}


fn main() {
    let args: Vec<String> = env::args().collect();
  
    let (screen_w, screen_h) = terminal::size().expect(
        "Can't get terminal size"
    );


    let mut config = match args.len() - 1 {
        0 => AppConfig::new(String::new(), screen_w, screen_h),
        1 => match args[1].as_str() {
            "-h" => {
                print_usage_message();
                return;
            },
            "--help" => {
                print_usage_message();
                return;
            },
            _ =>  AppConfig::new(args[1].clone(), screen_w, screen_h),
 

        },
                  _ => { 
            eprintln!("ERROR: Invalid number of arguments");
            print_usage_message();
            return;
        }
    };
    println!("Config: {} loaded!", config.name);

    let mut devices: Vec<Box<dyn Device>> = Vec::new();
    
    for tile in &config.tiles {
        if let Some(factory) = DEVICE_REGISTRY.get(tile.name.as_str()) {
           let device = (factory)(&tile);
           devices.push(device);
        }
        else {
            println!("Device {} not found in allowed list!", tile.name);
        }
    }

    let mut stdout = stdout();

    execute!(stdout, EnterAlternateScreen, cursor::Hide,).unwrap();

    enable_raw_mode().unwrap();


    loop {
        devices[0].update();
        let data = devices[0].show();
        for (ind, row) in data.iter().enumerate() {
            execute!(
                stdout, 
                MoveTo(0, ind as u16),
                Print(row)
            );
        }
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
