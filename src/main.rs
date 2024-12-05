use::std::env;

mod app_config;
mod file_config;
use crate::app_config::{AppConfig, DeviceTile};

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


fn print_usage_message() {
    println!("Usage: ");
    println!("tsm  [Options]");
    println!("Options:");
    println!("  <config_name>   Read config with the given name. Config must be placed in ~/.config/tsm/<config_name>.json");
}


fn main() {
    //read and generate app config
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

    // start main loop    
    let mut stdout = stdout();

    let mut ui = Ui::new(screen_w, screen_h);
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
