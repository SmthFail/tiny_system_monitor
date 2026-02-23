use tiny_tui::{
    app_error::AppError,
    buffer::{Buffer, Cell, Color, Patch},
    widget::{Rect, Widget},
    text_widget::TextWidget,
    cell_types::CellString,
    route::Route,
};

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

use crate::utils::version_checker::get_version;

struct ErrorRow {
    visible: bool,
    message: CellString,
    ui: Box<TextWidget>,
}

impl ErrorRow {
    fn new() -> Self {
        let message = CellString::new();
        let row =
            TextWidget::new_editable(message.clone()).set_color(Some(Color::White), Some(Color::Yellow));
        let ui = Box::new(row);

        ErrorRow { visible: false, message, ui }
    }

    fn set_message(&mut self, message: String) {
        self.visible = true;
        self.message.update(message);
    }

    fn render(&mut self, buffer: &mut Buffer, area: Rect) -> Result<(), AppError> {
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
    stdout: Stdout,

    // строки статуса/ошибок
    error_row: Box<ErrorRow>,
    status_row: Box<TextWidget>,

    // Route stack
    routes: Vec<Route>,
}

impl App {
    pub fn new() -> Result<Self, AppError> {
        let (width, height) =
            terminal::size().map_err(|e| AppError::error(format!("Can't get terminal size {}", e)))?;

        let version = get_version();
        let status_string = format!("q: exit, h: help, ver:{:?}", version);
        let status_row =
            TextWidget::new_static(&status_string).set_color(Some(Color::White), Some(Color::Blue));

        Ok(App {
            buffer: Buffer::new(width, height),
            prev_buffer: Buffer::new(width, height),
            stdout: stdout(),
            status_row: Box::new(status_row),
            error_row: Box::new(ErrorRow::new()),
            routes: Vec::new(),
        })
    }

    /// Add a route to the stack
    pub fn push_route(&mut self, route: Route) {
        self.routes.push(route);
    }

    /// Pop the top route from the stack
    pub fn pop_route(&mut self) -> Option<Route> {
        self.routes.pop()
    }

    /// Get the top route (for rendering/input)
    pub fn top_route(&mut self) -> Option<&mut Route> {
        self.routes.last_mut()
    }

    /// Get number of routes
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }

    /// Show help dialog as a route overlay
    pub fn show_help(&mut self) {
        use tiny_tui::engine::IntoNode;
        use tiny_tui::widgets::info_dialog::InfoDialog;

        let help_text = "q: quit\nh: toggle help";
        let dialog = InfoDialog::new("Help", help_text);

        let route = Route::new(vec![dialog.into_node()])
            .modal();

        self.push_route(route);
    }

    /// Hide help dialog (pop the top route if it's a help dialog)
    pub fn hide_help(&mut self) {
        // Only pop if top route is a help dialog (we could check by key or metadata)
        // For now, just pop if there's more than 1 route
        if self.routes.len() > 1 {
            self.pop_route();
        }
    }

    /// Toggle help dialog
    pub fn toggle_help(&mut self) {
        // Check if help dialog is already open (more than 1 route)
        if self.routes.len() > 1 {
            self.hide_help();
        } else {
            self.show_help();
        }
    }

    pub fn update(&mut self) -> Result<(), AppError> {
        self.status_row.update()?;
        Ok(())
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.buffer.resize(width, height);
        self.prev_buffer.resize(width, height);
        let _ = execute!(self.stdout, ResetColor, Clear(ClearType::All), cursor::MoveTo(0, 0));
    }

    pub fn render(&mut self) -> Result<(), AppError> {
        let buffer_width = self.buffer.width;
        let buffer_height = self.buffer.height;
        let body_height = if self.error_row.visible {
            buffer_height - 2
        } else {
            buffer_height - 1
        };

        let area = Rect { x: 0, y: 0, width: buffer_width, height: body_height };

        // Clear buffer to start fresh each frame
        // This ensures proper diff when routes are popped
        self.buffer.clear();

        // Render all routes (bottom to top)
        for route in &mut self.routes {
            route.render(&mut self.buffer, area)?;
        }

        // error row
        let _ = self.error_row.render(
            &mut self.buffer,
            Rect { x: 0, y: body_height, width: buffer_width, height: 1 },
        );

        // status row
        let status_y = buffer_height - 1;
        self.status_row.render(
            &mut self.buffer,
            Rect { x: 0, y: status_y, width: buffer_width, height: 1 },
        )?;

        Ok(())
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
            self.update()?;
            self.render()?;
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
                    Event::Key(KeyEvent {
                        code: KeyCode::Char('h'),
                        modifiers: KeyModifiers::NONE,
                        ..
                    }) => self.toggle_help(),
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
