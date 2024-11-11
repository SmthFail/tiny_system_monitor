mod cpu_info;
mod gpu_info;
mod ui;
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, execute, terminal};
use nvml_wrapper::error::NvmlError;
use std::io::{stdout, Write};
use std::time::Duration;

use crate::ui::LayoutBbox;
use ui::{LayoutType, Ui};

fn main() -> Result<(), NvmlError> {
    // init stdout, cpu and nvidia monitors
    let mut stdout = stdout();

    let (col, row) = terminal::size().expect("Can't get terminal size"); //TODO catch resize

    // init UI
    let mut ui = Ui::new(col, row);

    ui.create_layout(
        String::from("CPU_USAGE"),
        LayoutBbox {
            top: 0,
            left: 0,
            width: col / 2,
            height: row / 2,
        },
        LayoutType::Cpu,
    );
    ui.create_layout(
        String::from("GPU USAGE"),
        LayoutBbox {
            top: 0,
            left: col / 2 + 1,
            width: col / 2,
            height: row / 2,
        },
        LayoutType::Gpu,
    );

    execute!(stdout, EnterAlternateScreen, cursor::Hide,).unwrap();
    enable_raw_mode().unwrap();

    loop {
        ui.update_all();

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
                _ => (),
            }
        } else {
            // Timeout expired and no `Event` is available
        }
        stdout.flush().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(500)); //TODO check
    }
    disable_raw_mode().unwrap();
    Ok(())
}
