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


pub struct App {
    pub buffer: Buffer,
    children: Vec<Box<dyn Widget>>,
    stdout: Stdout 
}

impl App {
    pub fn new() -> Result<Self, AppError> {
        let (width, height) = terminal::size()
            .map_err(|e| AppError::error(format!("Can't get terminal size {}", e)))?;

        Ok(App {
            buffer: Buffer::new(width, height),
            children: Vec::new(),
            stdout: stdout()
        })
    }
    
    pub fn add_child(&mut self, child: Box<dyn Widget>) {
        self.children.push(child);
    }
    
    pub fn update(&mut self) {
        for child in &mut self.children {
            child.update()
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.buffer = Buffer::new(width, height);
        // TODO update size of child widget?
    }

    pub fn render(&mut self) {
        // check numbers of fixed lines and number of flexed widget 
        let mut fixed_lines: u16 = 0;
        let mut flex_widgets: u16 = 0;

        for child in &self.children {
            let (_, h) = child.get_constraints();
            match h {
                Some(h) => fixed_lines += h,
                None => flex_widgets += 1,
            } 
        }

        if fixed_lines + flex_widgets >= self.buffer.height{
           for (i, char) in "Overflowed".chars().enumerate() {
               self.buffer.set_cell(i as u16, 0, Cell::new(char));
           }
           return;
        }

        let mut current_row = 0;
        for child in &mut self.children {
            let (_, h) = child.get_constraints();
            let child_area = match h {
                Some(h) => Rect {
                    x: 0,
                    y: current_row,
                    width: self.buffer.width,
                    height: h
                },
                None => Rect {
                    x: 0,
                    y: current_row,
                    width: self.buffer.width,
                    height: (self.buffer.height - fixed_lines) / flex_widgets
                }
            };
            child.render(
                &mut self.buffer,
                child_area
                );
            current_row += child_area.height;
        }
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
        execute!(self.stdout, terminal::EnterAlternateScreen, cursor::Hide,).unwrap();

        terminal::enable_raw_mode().unwrap();

        loop {
            self.update();
            self.render();

            let _ = self.flush_to_terminal();

            if poll(Duration::from_millis(250)).unwrap() {
                match read().unwrap() {
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: KeyModifiers::NONE,
                        ..
                    }) => {
                        execute!(self.stdout, terminal::LeaveAlternateScreen, cursor::Show).unwrap();
                        break;
                    }
                    Event::Resize(width, height) => {
                        //queue!(stdout, terminal::Clear(terminal::ClearType::All)).unwrap();
                        self.resize(width, height);
                        
                    },
                    _ => (),
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(250));
            
        }
        terminal::disable_raw_mode().unwrap();
        Ok(()) 
   }

 
}
