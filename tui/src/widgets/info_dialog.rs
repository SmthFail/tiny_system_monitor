use crate::buffer::{Buffer, Cell, Color};
use crate::widget::{Widget, Rect, ChildConstraints};
use crate::app_error::AppError;
use crate::text_widget::TextWidget;
use crate::widgets::box_widget::BoxWidget;

/// A centered info dialog that displays a title and message
pub struct InfoDialog {
    title: String,
    message: String,
    is_visible: bool,
}

impl InfoDialog {
    pub fn new(title: &str, message: &str) -> Self {
        Self {
            title: title.to_string(),
            message: message.to_string(),
            is_visible: true,
        }
    }

    pub fn show(&mut self) {
        self.is_visible = true;
    }

    pub fn hide(&mut self) {
        self.is_visible = false;
    }

    pub fn toggle(&mut self) {
        self.is_visible = !self.is_visible;
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }
}

impl Widget for InfoDialog {
    fn render(&mut self, buf: &mut Buffer, area: Rect) -> Result<(), AppError> {
        if !self.is_visible {
            return Ok(());
        }

        // Calculate centered position
        let dialog_width = 30u16.min(area.width.saturating_sub(4));
        let dialog_height = 8u16.min(area.height.saturating_sub(4));
        
        let x = area.x + (area.width - dialog_width) / 2;
        let y = area.y + (area.height - dialog_height) / 2;

        let dialog_area = Rect {
            x,
            y,
            width: dialog_width,
            height: dialog_height,
        };

        // Fill background with black to cover content below
        for dy in 0..dialog_area.height {
            for dx in 0..dialog_area.width {
                let cell = Cell::new(' ').bg(Some(Color::Black));
                buf.set_cell(dialog_area.x + dx, dialog_area.y + dy, cell);
            }
        }

        // Create and render the boxed content
        let mut box_widget = BoxWidget::new()
            .border(true)
            .name(self.title.clone())
            .child(TextWidget::new_static(&self.message).max_lines(dialog_height.saturating_sub(2)));

        box_widget.render(buf, dialog_area)
    }

    fn get_constraints(&self) -> ChildConstraints {
        if self.is_visible {
            ChildConstraints {
                min_width: Some(20),
                max_width: Some(40),
                min_height: Some(5),
                max_height: Some(15),
            }
        } else {
            ChildConstraints {
                min_width: Some(0),
                max_width: Some(0),
                min_height: Some(0),
                max_height: Some(0),
            }
        }
    }

    fn update(&mut self) -> Result<(), AppError> {
        Ok(())
    }
}
