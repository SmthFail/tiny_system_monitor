mod app_config;
mod file_config;

use std::io::{stdout, Write};

use crossterm::event::{poll, read, Event, KeyEvent, KeyCode, KeyModifiers};
use crossterm::{cursor, execute, queue};
use crossterm::cursor::MoveTo;
use crossterm::style::Print;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen
};
use text_widget::TextWidget;

use std::time::Duration;
mod cpu_info;
mod gpu_info;
mod ui;
//mod device_model;
//use device_model::{Device, DEVICE_REGISTRY};
//mod cpu_device;
//mod gpu_device;

mod widget;
//use widget::Rect;

mod buffer;
use buffer::Buffer;

mod text_widget;
mod progress_widget;

mod container_widget;
use container_widget::{Container, Layout, Alignment};

mod app;
use app::App;

fn print_usage_message() {
    println!("Usage: ");
    println!("tsm  [Options]");
    println!("Options:");
    println!("  <config_name>   Read config with the given name. Config must be placed in ~/.config/tsm/<config_name>.json");
    println!("  -h, --help      Print help message")
}


pub fn flush_buffer_to_crossterm(buffer: &Buffer) -> crossterm::Result<()> {
    let mut stdout = stdout();


    for y in 0..buffer.height {
        for x in 0..buffer.width {
            let idx = (y as usize) * (buffer.width as usize) + (x as usize);
            let cell = &buffer.cells[idx];
            
            // Двигаем курсор в (x, y)
            queue!(stdout, MoveTo(x, y))?;
           queue!(stdout, Print(cell.symbol))?;
        }
    }

    stdout.flush()?;
    Ok(())
}


fn main() {
    let mut stdout = stdout();
    
    let mut app = App::new();

    // main container 
    let mut tile = Container::new(None, None, Layout::Horizontal, Alignment::Start, false);
    tile.add_child(Box::new(cpu_info::CpuInfo::new()));
    tile.add_child(Box::new(gpu_info::GpuInfo::new()));

    app.add_child(Box::new(tile));

    let version = env!("CARGO_PKG_VERSION");
    let status_string = format!("q: exit, ver: {}", version);
    app.add_child(Box::new(TextWidget::new(&status_string)));
    
    execute!(stdout, EnterAlternateScreen, cursor::Hide,).unwrap();

    enable_raw_mode().unwrap();

    loop {
        app.update();
        app.render();

        let _ = flush_buffer_to_crossterm(&app.buffer);

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
                    //queue!(stdout, terminal::Clear(terminal::ClearType::All)).unwrap();
                    app.resize(width, height);
                    
                },
                _ => (),
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
        
    }
    disable_raw_mode().unwrap();
}
