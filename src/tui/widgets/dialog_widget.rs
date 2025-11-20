use crate::tui::buffer::Buffer;
use crate::tui::widget::{Widget, Rect, ChildConstraints};
use crate::tui::app_error::AppError;
use crate::tui::text_widget::TextWidget;
use crate::tui::widgets::box_widget::BoxWidget;

pub struct DialogWidget {
    box_widget: BoxWidget,
    is_visible: bool,
}

impl DialogWidget {
    pub fn new(title: &str, message: &str) -> Self {
        let text_widget = TextWidget::new_static(message)
            .max_lines(10); // Allow up to 10 lines for dialog content

        let box_widget = BoxWidget::new()
            .border(true)
            .name(String::from(title))
            .child(text_widget);

        Self {
            box_widget,
            is_visible: false,
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

impl Widget for DialogWidget {
    fn render(&mut self, buf: &mut Buffer, area: Rect) -> Result<(), AppError> {
        if !self.is_visible {
            return Ok(());
        }

        // Calculate centered position for the dialog - make it smaller for simple help text
        let dialog_width = std::cmp::min(25u16, area.width.saturating_sub(2));  // Small width, max available width - 2
        let dialog_height = std::cmp::min(5u16, area.height.saturating_sub(2)); // Small height, max available height - 2
        let x = area.x + (area.width - dialog_width) / 2;
        let y = area.y + (area.height - dialog_height) / 2;

        let dialog_area = Rect {
            x,
            y,
            width: dialog_width,
            height: dialog_height,
        };

        // First, fill the entire dialog area with a background to ensure no artifacts remain
        for y_pos in dialog_area.y..(dialog_area.y + dialog_area.height) {
            for x_pos in dialog_area.x..(dialog_area.x + dialog_area.width) {
                if x_pos < buf.width && y_pos < buf.height {
                    let cell = crate::tui::buffer::Cell::new(' ').bg(Some(crate::tui::buffer::Color::Black));
                    buf.set_cell(x_pos, y_pos, cell);
                }
            }
        }

        // Then render the box widget
        self.box_widget.render(buf, dialog_area)
    }

    fn get_constraints(&self) -> ChildConstraints {
        if self.is_visible {
            // When visible, set more reasonable constraints for a dialog
            ChildConstraints {
                min_width: Some(20),
                max_width: Some(60),
                min_height: Some(5),
                max_height: Some(15),
            }
        } else {
            // When hidden, it doesn't require any space
            ChildConstraints {
                min_width: Some(0),
                max_width: Some(0),
                min_height: Some(0),
                max_height: Some(0),
            }
        }
    }

    fn update(&mut self) -> Result<(), AppError> {
        if self.is_visible {
            self.box_widget.update()
        } else {
            Ok(())
        }
    }
}