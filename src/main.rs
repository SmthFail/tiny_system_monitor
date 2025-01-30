use::std::env;
mod app_config;
mod file_config;
use crate::app_config::AppConfig;

use std::io::{stdout, Write};

use crossterm::event::{poll, read, Event, KeyEvent, KeyCode, KeyModifiers};
use crossterm::{cursor, execute, queue};
use crossterm::cursor::MoveTo;
use crossterm::style::{Print, ResetColor, Color, SetForegroundColor};
use crossterm::terminal::{
    self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen
};
use progress_widget::ProgressBar;
use text_widget::TextWidget;

use std::time::Duration;

mod cpu_info;
mod gpu_info;
mod ui;
mod device_model;
use device_model::{Device, DEVICE_REGISTRY};
mod cpu_device;
mod gpu_device;

mod widget;
use widget::Rect;

mod buffer;
use buffer::{Buffer};

mod text_widget;
mod progress_widget;
use crate::widget::Widget;

mod tile_widget;
use tile_widget::{Tile, Layout};

fn print_usage_message() {
    println!("Usage: ");
    println!("tsm  [Options]");
    println!("Options:");
    println!("  <config_name>   Read config with the given name. Config must be placed in ~/.config/tsm/<config_name>.json");
    println!("  -h, --help      Print help message")
}


pub fn flush_buffer_to_crossterm(buffer: &Buffer) -> crossterm::Result<()> {
    let mut stdout = stdout();

    // Для примера — просто бежим по всем ячейкам и выводим символы.
    // Можно оптимизировать, обновляя только те ячейки, что изменились с прошлого кадра.

    let mut last_fg: Option<Color> = None;
    let mut last_bg: Option<Color> = None;

    for y in 0..buffer.height {
        for x in 0..buffer.width {
            let idx = (y as usize) * (buffer.width as usize) + (x as usize);
            let cell = &buffer.cells[idx];
            
            // Двигаем курсор в (x, y)
            queue!(stdout, MoveTo(x, y))?;
            /*
            // Устанавливаем цвет перед тем, как вывести символ (если отличается от предыдущего)
            if cell.fg != last_fg {
                if let Some(fg_color) = &cell.fg {
                    queue!(stdout, SetForegroundColor(to_crossterm_color(fg_color)))?;
                } else {
                    // Сброс цвета
                    queue!(stdout, SetForegroundColor(CtColor::Reset))?;
                }
                last_fg = cell.fg.clone();
            }
            if cell.bg != last_bg {
                if let Some(bg_color) = &cell.bg {
                    queue!(stdout, SetBackgroundColor(to_crossterm_color(bg_color)))?;
                } else {
                    // Сброс фона
                    queue!(stdout, SetBackgroundColor(CtColor::Reset))?;
                }
                last_bg = cell.bg.clone();
            }
            */
            // Выводим символ
            queue!(stdout, Print(cell.symbol))?;
        }
    }

    // Сбрасываем цвета в конце (по желанию)
    // queue!(stdout, SetForegroundColor(CtColor::Reset))?;
    // queue!(stdout, SetBackgroundColor(CtColor::Reset))?;

    stdout.flush()?;
    Ok(())
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
    /*
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
*/
    let mut stdout = stdout();
    let mut buffer = Buffer::new(screen_w, screen_h);
    
    // for test
    let caption = TextWidget::new("This is own capture!");

    let mut tile = Tile::new(screen_w, screen_h, Layout::Vertical);
    tile.add_child(Box::new(TextWidget::new("First")));
    tile.add_child(Box::new(TextWidget::new("Second")));
    tile.add_child(Box::new(ProgressBar::new("Ram", "Mb")));
    
    execute!(stdout, EnterAlternateScreen, cursor::Hide,).unwrap();

    enable_raw_mode().unwrap();

    tile.update();
    loop {
        /*
        for device in &mut devices {
            device.update();
            let position = device.get_position();
            for (ind, row) in device.show().iter().enumerate() {
                queue!(
                    stdout,
                    MoveTo(position.1, ind as u16 + position.0),
                    Print(row)
                ).unwrap(); 
            }
        } 
        */
        caption.render(&mut buffer, Rect{
            x: 0, y: 0, width: screen_w, height: 1,
        });

        tile.render(
            &mut buffer,
            Rect {
                x: 0,
                y: 4,
                width: screen_w,
                height: screen_h - 4
            }
        );

        let _ = flush_buffer_to_crossterm(&buffer);

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

                    screen_w = width;
                    screen_h = height;
                    
                    buffer = Buffer::new(width, height);
                    tile.width = width;
                    tile.height = height - 4;
                    
                },
                _ => (),
            }
        }
        /*
        queue!(
            stdout, 
            MoveTo(0, screen_h), 
            SetForegroundColor(Color::Green),
            Print(format!("q: exit, config: {}", config.name)),
            ResetColor).unwrap();
        stdout.flush().unwrap();
        */
                std::thread::sleep(std::time::Duration::from_millis(250));
        
    }
    disable_raw_mode().unwrap();
}
