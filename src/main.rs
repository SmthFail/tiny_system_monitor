use::std::env;
mod app_config;
mod file_config;
use crate::app_config::{AppConfig, DeviceTile};

use std::io::{stdout, Write};

use crossterm::event::{poll, read, Event, KeyEvent, KeyCode, KeyModifiers};
use crossterm::{cursor, execute, queue};
use crossterm::cursor::{MoveTo, MoveToRow};
use crossterm::style::{Print, ResetColor, Color, SetForegroundColor};
use crossterm::terminal::{
    self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen
};

use std::time::Duration;

mod cpu_info;
mod gpu_info;
mod ui;
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
  
    let (mut screen_w, mut screen_h) = terminal::size().expect(
        "Can't get terminal size"
    );

    // leave last line for information
    screen_h -= 1;
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

    let mut devices: Vec<Box<dyn Device>> = Vec::new();
    
    for tile in &config.tiles {
        if let Some(factory) = DEVICE_REGISTRY.get(tile.name.as_str()) {
           let device = (factory)(tile);
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
        for device in &mut devices {
            device.update();
        } 

        for device in &mut devices {
            let position = device.get_position();
            for (ind, row) in device.show().iter().enumerate() {
                queue!(
                    stdout,
                    MoveTo(position.1, ind as u16 + position.0),
                    Print(row)
                ).unwrap(); 
            }
        } 

        if poll(Duration::from_millis(250)).unwrap() {
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
                    queue!(stdout, terminal::Clear(terminal::ClearType::All)).unwrap();
                    
                    for tile in &config.tiles {
                        for device in &mut devices {
                            if device.get_name() == tile.name {
                                device.resize(tile.width, tile.height);
                            }
                        }
                    }

                    screen_w = width;
                    screen_h = height;
                    config.update_grid(width, height);
                },
                _ => (),
            }
        }

        queue!(
            stdout, 
            MoveTo(0, screen_h), 
            SetForegroundColor(Color::Green),
            Print(format!("q: exit, config: {}", config.name)),
            ResetColor).unwrap();
        stdout.flush().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(250));
        
    }
    disable_raw_mode().unwrap();

}
