use super::buffer::{Buffer, Color};
use super::widget::{Widget, Rect};
use crossterm::terminal;
use std::io::{stdout, Stdout, Write};
use std::{
    rc::Rc,
    cell::RefCell
};
use crossterm::event::{poll, read, Event, KeyEvent, KeyCode, KeyModifiers};
use std::time::Duration;
use crossterm::style::{
    SetForegroundColor, 
    SetBackgroundColor,
    Print, ResetColor, Color as CrossColor};
use crossterm::{cursor, execute, queue};
use crossterm::cursor::MoveTo;
use super::app_error::AppError;

use crate::{
    Container,
    Layout,
    Alignment,
    TextWidget,
    get_version
};



struct WarningRow {
    is_error: bool,
    message: Rc<RefCell<String>>,
    ui: Box<TextWidget>
}

impl WarningRow {
    fn new() -> Self {
        let message = Rc::new(RefCell::new(String::new()));
        
        let row = TextWidget::new_editable(message.clone())
            .set_color(Some(Color::White), Some(Color::Yellow));
        let ui = Box::new(row);

        WarningRow {
            is_error: false,
            message,
            ui
        }
    }

    fn update(&mut self, is_error: bool, message: String) {
        self.is_error = is_error;
        *self.message.borrow_mut() = message;
    }

    fn render(&mut self, buffer: &mut Buffer, area: Rect) -> Result<(), AppError>{
        self.ui.render(buffer, area)?;
        Ok(())
    }

    fn clear(&mut self) {
        self.is_error = false;
        *self.message.borrow_mut() = String::new();
    }
}

pub struct App {
    pub buffer: Buffer,
    body: Box<Container>,
    stdout: Stdout,
    warning_status: WarningRow,
    status_row: Box<TextWidget>
}

impl App {
    pub fn new() -> Result<Self, AppError> {
        let (width, height) = terminal::size()
            .map_err(|e| AppError::error(format!("Can't get terminal size {}", e)))?;

        let body = Container::new(None,None, Layout::Vertical, Alignment::Start, false);

        // generate status row
        let version = get_version();
        let status_string = format!("q: exit, ver:{:?}", version);
        let status_row = TextWidget::new_static(&status_string).set_color(Some(Color::White), Some(Color::Blue));

        Ok(App {
            buffer: Buffer::new(width, height),
            body: Box::new(body),
            stdout: stdout(),
            status_row: Box::new(status_row),
            warning_status: WarningRow::new() 
        })
    }
    
    pub fn add_child(&mut self, child: Box<dyn Widget>) {
        self.body.add_child(child);
    }
    
    pub fn update(&mut self) {
        self.body.update();
        self.status_row.update();
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.buffer = Buffer::new(width, height);
    }

    pub fn render(&mut self) -> Result<(), AppError>{
        let width = self.buffer.width;
        let mut body_height = if self.warning_status.is_error {
            self.buffer.height - 2
        } else {
            self.buffer.height - 1
        };

        let body_render =  self.body.render(
            &mut self.buffer,
            Rect {
               x: 0,
               y: 0,
               width: width,
               height: body_height, 
            }
        );
        match body_render {
            Ok(_) => {
                self.warning_status.clear();
            },
            Err(value) => {
                self.warning_status.update(true, value.message);
            }
        }

        if self.warning_status.is_error {
            let _ = self.warning_status.render(
                &mut self.buffer,
                Rect {
                    x: 0,
                    y: body_height,
                    width: width,
                    height: 1
                }
            );
            body_height += 1;
        }

        self.status_row.render(
            &mut self.buffer,
            Rect {
                x: 0,
                y: body_height,
                width: width,
                height: 1
            }
            )?;

        Ok(())
    }
    

    fn match_cross_color(color: &Color) -> CrossColor {
        let new_color = match color {
            Color::Red => CrossColor::Red,
            Color::Green => CrossColor::Green,
            Color::Blue => CrossColor::Blue,
            Color::White => CrossColor::White,
            Color::Black => CrossColor::Black,
            Color::Yellow => CrossColor::DarkYellow
        };
        new_color
    }

    fn flush_to_terminal(&mut self) -> crossterm::Result<()> {
        for y in 0..self.buffer.height {
            for x in 0..self.buffer.width {
                let idx = (y as usize) * (self.buffer.width as usize) + (x as usize);
                let cell = &self.buffer.cells[idx];
                
                queue!(self.stdout, MoveTo(x, y))?;
                match &cell.fg {
                    Some(color) => {
                        let fg = Self::match_cross_color(&color);
                        queue!(self.stdout, SetForegroundColor(fg))?;
                    },
                    None => {}
                };
                match &cell.bg {
                    Some(color) => {
                        let bg = Self::match_cross_color(&color);
                        queue!(self.stdout, SetBackgroundColor(bg))?;
                    },
                    None => {}
                }
                queue!(self.stdout, Print(cell.symbol), ResetColor)?;
            }
        }

        self.stdout.flush()?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), AppError> {
        execute!(self.stdout, terminal::EnterAlternateScreen, cursor::Hide,)
            .map_err(|e| AppError::error(format!("Can't enter alternate screen: {}", e)))?;

        terminal::enable_raw_mode()
            .map_err(|e| AppError::error(format!("Can't enable_raw_mode: {}", e)))?;

        loop {
            self.update();
            self.render()?;

            let _ = self.flush_to_terminal();

            if poll(Duration::from_millis(500))
                .map_err(|e| AppError::error(format!("Poll error: {}", e)))? {

                let event = read().map_err(|e| AppError::error(format!("Event read error: {}", e)))?;
                match event {
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: KeyModifiers::NONE,
                        ..
                    }) => {
                        break;
                    }
                    Event::Resize(width, height) => {
                        self.resize(width, height);
                    },
                    _ => (),
                }
            }
        }
        terminal::disable_raw_mode()
            .map_err(|e| AppError::error(format!("Can't disable raw mode: {}", e)))?;

        execute!(self.stdout, terminal::LeaveAlternateScreen, cursor::Show)
            .map_err(|e| AppError::error(format!("Can't leave alternate screen: {}", e)))?;
        
        Ok(()) 
    }
}
