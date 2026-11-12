use crate::app::App;
use crate::app_error::AppError;
use crate::buffer::{Buffer, Cell, Color, Patch};

use crossterm::{
    cursor,
    execute,
    queue,
    style::{Color as CrossColor, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal,
    terminal::{Clear, ClearType},
};
use crossterm::event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers};
use std::io::{stdout, Stdout, Write};
use std::mem;
use std::time::Duration;

/// Owns the terminal: stdout, the front/back buffers, alternate screen + raw
/// mode, event polling, resize plumbing and the frame loop. The widget side
/// (route stack, status row) lives in `crate::app::App`.
pub struct Session {
    app: App,
    buffer: Buffer,
    prev_buffer: Buffer,
    stdout: Stdout,
}

impl Session {
    pub fn new(app: App) -> Result<Self, AppError> {
        let (width, height) =
            terminal::size().map_err(|e| AppError::error(format!("Can't get terminal size {}", e)))?;

        Ok(Session {
            app,
            buffer: Buffer::new(width, height),
            prev_buffer: Buffer::new(width, height),
            stdout: stdout(),
        })
    }

    fn resize(&mut self, width: u16, height: u16) {
        self.buffer.resize(width, height);
        self.prev_buffer.resize(width, height);
        self.app.resize(width, height);
        let _ = execute!(self.stdout, ResetColor, Clear(ClearType::All), cursor::MoveTo(0, 0));
    }

    fn match_cross_color(color: &Color) -> CrossColor {
        match color {
            Color::Red => CrossColor::Red,
            Color::Green => CrossColor::Green,
            Color::Blue => CrossColor::Blue,
            Color::White => CrossColor::White,
            Color::Black => CrossColor::Black,
            Color::Yellow => CrossColor::DarkYellow,
        }
    }

    fn flush_to_terminal(&mut self) -> crossterm::Result<()> {
        queue!(self.stdout, ResetColor)?;

        let mut cur_fg: Option<Color> = None;
        let mut cur_bg: Option<Color> = None;

        let patches = self.prev_buffer.get_diff(&self.buffer);
        for Patch { cell: Cell { ch, fg, bg }, x, y } in patches {
            queue!(self.stdout, cursor::MoveTo(x as u16, y as u16))?;

            if fg != cur_fg {
                match &fg {
                    Some(color) => queue!(
                        self.stdout,
                        SetForegroundColor(Self::match_cross_color(&color))
                    )?,
                    None => queue!(self.stdout, SetForegroundColor(CrossColor::Reset))?,
                }
                cur_fg = fg.clone();
            }

            if bg != cur_bg {
                match &bg {
                    Some(color) => queue!(
                        self.stdout,
                        SetBackgroundColor(Self::match_cross_color(&color))
                    )?,
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
        execute!(self.stdout, terminal::EnterAlternateScreen, cursor::Hide)
            .map_err(|e| AppError::error(format!("Can't enter alternate screen: {}", e)))?;
        terminal::enable_raw_mode()
            .map_err(|e| AppError::error(format!("Can't enable_raw_mode: {}", e)))?;

        loop {
            self.app.update()?;
            let size = (self.buffer.width, self.buffer.height);
            self.app.render(&mut self.buffer, size)?;
            let _ = self.flush_to_terminal();

            if poll(Duration::from_millis(500))
                .map_err(|e| AppError::error(format!("Poll error: {}", e)))?
            {
                let event = read().map_err(|e| AppError::error(format!("Event read error: {}", e)))?;
                match event {
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: KeyModifiers::NONE,
                        ..
                    }) => break,
                    Event::Resize(width, height) => self.resize(width, height),
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
