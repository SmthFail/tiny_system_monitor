use crate::{
    app_error::AppError,
    buffer::{Buffer, Color},
    widget::{Rect, Widget},
    widgets::text_widget::TextWidget,
    cell_types::CellString,
    route::Route,
};

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

/// Top-level widget: owns the route stack and the error/status rows.
/// Terminal ownership (stdout, buffers, raw mode, the event loop) lives in
/// `crate::session::Session`.
pub struct App {
    // строки статуса/ошибок
    error_row: Box<ErrorRow>,
    status_row: Box<TextWidget>,

    // Route stack
    routes: Vec<Route>,
}

impl App {
    pub fn new(version: String) -> Self {
        let status_string = format!("q: exit, ver:{:?}", version);
        let status_row =
            TextWidget::new_static(&status_string).set_color(Some(Color::White), Some(Color::Blue));

        App {
            status_row: Box::new(status_row),
            error_row: Box::new(ErrorRow::new()),
            routes: Vec::new(),
        }
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

    pub fn update(&mut self) -> Result<(), AppError> {
        self.status_row.update()?;
        Ok(())
    }

    /// Resize the route rects. Routes are laid out from the render area on
    /// every frame, so there is nothing stored to resize today.
    pub fn resize(&mut self, _width: u16, _height: u16) {}

    /// Paint the route stack, error row and status row into a
    /// caller-provided buffer of the given size.
    pub fn render(&mut self, buffer: &mut Buffer, size: (u16, u16)) -> Result<(), AppError> {
        let (buffer_width, buffer_height) = size;
        let body_height = if self.error_row.visible {
            buffer_height - 2
        } else {
            buffer_height - 1
        };

        let area = Rect { x: 0, y: 0, width: buffer_width, height: body_height };

        // Clear buffer to start fresh each frame
        // This ensures proper diff when routes are popped
        buffer.clear();

        // Render all routes (bottom to top)
        for route in &mut self.routes {
            route.render(buffer, area)?;
        }

        // error row
        let _ = self.error_row.render(
            buffer,
            Rect { x: 0, y: body_height, width: buffer_width, height: 1 },
        );

        // status row
        let status_y = buffer_height - 1;
        self.status_row.render(
            buffer,
            Rect { x: 0, y: status_y, width: buffer_width, height: 1 },
        )?;

        Ok(())
    }
}
