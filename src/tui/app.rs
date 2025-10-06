use super::buffer::{Buffer, Color, Cell, Patch};
use super::widget::{Widget, Rect};
use std::io::{stdout, Stdout, Write};
use crossterm::event::{poll, read, Event, KeyEvent, KeyCode, KeyModifiers};
use std::time::Duration;
use crossterm::{
    cursor,
    terminal, 
    style::{
        SetForegroundColor, 
        SetBackgroundColor,
        Print, ResetColor, Color as CrossColor
    },
    terminal::{Clear, ClearType},
    execute, 
    queue};

use super::app_error::AppError;

use crate::{
    Container,
    Layout,
    Alignment,
    TextWidget,
    get_version,
};
use crate::tui::cell_types::CellString;

use std::mem;

struct ErrorRow {
    visible: bool,
    message: CellString,
    ui: Box<TextWidget>
}

impl ErrorRow {
    fn new() -> Self {
        let message = CellString::new();
        
        let row = TextWidget::new_editable(message.clone())
            .set_color(Some(Color::White), Some(Color::Yellow));
        let ui = Box::new(row);

        ErrorRow {
            visible: false,
            message,
            ui
        }
    }

    fn set_message(&mut self, message: String) {
        self.visible = true;
        self.message.update(message);
    }

    fn render(&mut self, buffer: &mut Buffer, area: Rect) -> Result<(), AppError>{
        if self.visible {
            self.ui.render(buffer, area)?;
        }
        Ok(())
    }

    fn clear(&mut self) {
        self.message.clear();
        self.visible = false;
    }
}

pub struct App {
    pub buffer: Buffer,
    prev_buffer: Buffer,
    pub body: Box<Container>,
    stdout: Stdout,
    error_row: Box<ErrorRow>,
    status_row: Box<TextWidget>
}

impl App {
    pub fn new() -> Result<Self, AppError> {
        let (width, height) = terminal::size()
            .map_err(|e| AppError::error(format!("Can't get terminal size {}", e)))?;
        let body = Container::new(None,None, Layout::Vertical, Alignment::Start);

        // generate status row
        let version = get_version();
        let status_string = format!("q: exit, ver:{:?}", version);
        let status_row = TextWidget::new_static(&status_string)
            .set_color(Some(Color::White), Some(Color::Blue));
        
        // error row
        let error_row = ErrorRow::new();

        Ok(App {
            buffer: Buffer::new(width, height),
            prev_buffer: Buffer::new(width, height),
            body: Box::new(body),
            stdout: stdout(),
            status_row: Box::new(status_row),
            error_row: Box::new(error_row)
        })
    }
    
    pub fn add_child(&mut self, child: Box<dyn Widget>) {
        self.body.add_child(child);
    }
    
    pub fn update(&mut self) -> Result<(), AppError> {
        self.body.update()?;
        self.status_row.update()?;
        Ok(())
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.buffer.resize(width, height);
        self.prev_buffer.resize(width, height);
        let _ = execute!(self.stdout, ResetColor, Clear(ClearType::All), cursor::MoveTo(0, 0));
    }

    pub fn render(&mut self) -> Result<(), AppError>{
        let width = self.buffer.width;
        let body_height = if self.error_row.visible {
            self.buffer.height - 2
        } else {
            self.buffer.height - 1
        };

        let body_render =  self.body.render(
            &mut self.buffer,
            Rect {
               x: 0,
               y: 0,
               width,
               height: body_height, 
            }
        );
        match body_render {
            Ok(_) => {
                self.error_row.clear();
            },
            Err(value) => {
                self.error_row.set_message(value.message);
            }
        }
        
        let _ = self.error_row.render(
            &mut self.buffer,
            Rect {
                x: 0,
                y: body_height,
                width,
                height: 1
            }
        );

        let status_position = self.buffer.height - 1;
        self.status_row.render(
            &mut self.buffer,
            Rect {
                x: 0,
                y: status_position,
                width,
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
        queue!(self.stdout, ResetColor)?;

        let mut cur_fg: Option<Color> = None;
        let mut cur_bg: Option<Color> = None;
        
        let patches = self.prev_buffer.get_diff(&self.buffer);
        for Patch {cell: Cell {ch, fg, bg}, x, y} in patches {
                
                queue!(self.stdout, cursor::MoveTo(x as u16, y as u16))?;
                
                if fg != cur_fg {
                    match &fg {
                        Some(color) => queue!(
                            self.stdout, 
                            SetForegroundColor(Self::match_cross_color(&color)
                        ))?,
                        None => queue!(self.stdout, SetForegroundColor(CrossColor::Reset))?,
                    }
                    cur_fg = fg.clone();
                }
                
                if bg != cur_bg {
                    match &bg {
                        Some(color) => queue!(
                            self.stdout, 
                            SetBackgroundColor(Self::match_cross_color(&color)
                        ))?,
                        None => queue!(self.stdout, SetBackgroundColor(CrossColor::Reset))?,
                    }
                    cur_bg = bg.clone();
                }

                queue!(self.stdout, Print(ch))?;
        }

        self.stdout.flush()?;
        
        mem::swap(&mut self.buffer, &mut self.prev_buffer);
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), AppError> {
        execute!(self.stdout, terminal::EnterAlternateScreen, cursor::Hide,)
            .map_err(|e| AppError::error(format!("Can't enter alternate screen: {}", e)))?;

        terminal::enable_raw_mode()
            .map_err(|e| AppError::error(format!("Can't enable_raw_mode: {}", e)))?;

        loop {
            self.update()?;
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
