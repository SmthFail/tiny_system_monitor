use crate::tui::buffer::{Buffer, Color};
use crate::tui::widget::{Widget, Rect, ChildConstraints};
use crate::tui::app_error::AppError;
use crate::tui::text_widget::TextWidget;

pub struct ErrorWidget {
    message: String,
}

impl ErrorWidget {
    pub fn new(error: &AppError) -> Self {
        Self::new_with_message(&error.message)
    }

    pub fn new_with_message(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl Widget for ErrorWidget {
    fn render(&mut self, buf: &mut Buffer, area: Rect) -> Result<(), AppError> {
        // Display error message in the full area (the parent BoxWidget handles the border)
        let mut text_widget = TextWidget::new_static(&self.message)
            .set_color(Some(Color::Red), Some(Color::Black))
            .max_lines(area.height);

        text_widget.render(buf, area)?;

        Ok(())
    }

    fn get_constraints(&self) -> ChildConstraints {
        ChildConstraints {
            min_width: Some(5),  // Minimum width to show a short error message
            max_width: None,
            min_height: Some(1), // Minimum height to show text
            max_height: None,
        }
    }
}