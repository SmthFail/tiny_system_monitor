use super::buffer::{Buffer, Cell, Color};
use super::widget::{Widget, Rect};
use crossterm::terminal;
use std::io::{stdout, Stdout, Write};
use crossterm::event::{poll, read, Event, KeyEvent, KeyCode, KeyModifiers};
use std::time::Duration;
use crossterm::style::{SetForegroundColor, Print, ResetColor, Color as CrossColor};
use crossterm::{cursor, execute, queue};
use crossterm::cursor::MoveTo;
use super::app_error::AppError;

use crate::Container;
use crate::Layout;
use crate::Alignment;
use crate::TextWidget;

use crate::get_version; 


pub struct App {
    pub buffer: Buffer,
    children: Box<Container>,
    stdout: Stdout,
    width: u16,
    height: u16,
    status_row: Box<TextWidget>
}

impl App {
    pub fn new() -> Result<Self, AppError> {
        let (width, height) = terminal::size()
            .map_err(|e| AppError::error(format!("Can't get terminal size {}", e)))?;

        let children = Container::new(None,None, Layout::Vertical, Alignment::Start, false);

        // generate status row
        let version = get_version();
        let status_string = format!("q: exit, ver:{:?}", version);
        let status_row = TextWidget::new(&status_string);
        Ok(App {
            buffer: Buffer::new(width, height),
            children: Box::new(children),
            stdout: stdout(),
            width,
            height,
            status_row: Box::new(status_row)
        })
    }
    
    pub fn add_child(&mut self, child: Box<dyn Widget>) {
        self.children.add_child(child);
    }
    
    pub fn update(&mut self) {
        self.children.update();
        self.status_row.update();
    }

    pub fn resize(&mut self) {
        self.buffer = Buffer::new(self.width, self.height);
        // TODO update size of child widget?
    }

    pub fn render(&mut self) {
        self.children.render(
            &mut self.buffer,
            Rect {
               x: 0,
               y: 0,
               width: self.width,
               height: self.height - 1, 
            }
        );
        self.status_row.render(
            &mut self.buffer,
            Rect {
                x: 0,
                y: self.height - 1,
                width: self.width,
                height: 1
            }
            );
    }

    fn flush_to_terminal(&mut self) -> crossterm::Result<()> {
        for y in 0..self.buffer.height {
            for x in 0..self.buffer.width {
                let idx = (y as usize) * (self.buffer.width as usize) + (x as usize);
                let cell = &self.buffer.cells[idx];
                
                queue!(self.stdout, MoveTo(x, y))?;
                match &cell.fg {
                    Some(color) => {
                        let fg = match color {
                            Color::Red => CrossColor::Red,
                            Color::Green => CrossColor::Green,
                            Color::Blue => CrossColor::Blue,
                            Color::White => CrossColor::White,
                            Color::Black => CrossColor::Black,
                            Color::Yellow => CrossColor::DarkYellow
                        };
                        queue!(
                            self.stdout, SetForegroundColor(fg),
                            Print(cell.symbol),
                            ResetColor
                        )?
                    },
                    None => queue!(self.stdout, Print(cell.symbol))?
                }
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
            self.render();

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
                        self.width = width;
                        self.height = height;
                        self.resize();
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
