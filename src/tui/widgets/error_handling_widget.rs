use crate::tui::buffer::Buffer;
use crate::tui::widget::{Widget, Rect, ChildConstraints};
use crate::tui::app_error::AppError;
use crate::tui::widgets::error_widget::ErrorWidget;

pub struct ErrorHandlingWidget {
    widget: Box<dyn Widget>,
    error_widget: Option<Box<dyn Widget>>,
    is_error_state: bool,
    last_error: Option<AppError>,
}

impl ErrorHandlingWidget {
    pub fn new(widget: impl Widget + 'static) -> Self {
        Self {
            widget: Box::new(widget),
            error_widget: None,
            is_error_state: false,
            last_error: None,
        }
    }

    pub fn with_error_widget(mut self, error_widget: impl Widget + 'static) -> Self {
        self.error_widget = Some(Box::new(error_widget));
        self
    }
}

impl Widget for ErrorHandlingWidget {
    fn render(&mut self, buf: &mut Buffer, area: Rect) -> Result<(), AppError> {
        if self.is_error_state {
            // Try to render the error widget, fallback to a default error widget if needed
            if let Some(ref mut error_widget) = self.error_widget {
                return error_widget.render(buf, area);
            } else if let Some(ref last_error) = self.last_error {
                let mut default_error_widget = ErrorWidget::new(last_error);
                return default_error_widget.render(buf, area);
            } else {
                // Fallback: create a generic error widget
                let mut fallback_error_widget = ErrorWidget::new_with_message("An unknown error occurred");
                return fallback_error_widget.render(buf, area);
            }
        } else {
            // Try to render the main widget
            match self.widget.render(buf, area) {
                Ok(()) => Ok(()),
                Err(e) => {
                    // Switch to error state
                    self.is_error_state = true;
                    self.last_error = Some(e.clone());
                    // Render the error widget
                    if let Some(ref mut error_widget) = self.error_widget {
                        error_widget.render(buf, area)
                    } else {
                        let mut error_widget = ErrorWidget::new(&e);
                        error_widget.render(buf, area)
                    }
                }
            }
        }
    }

    fn get_constraints(&self) -> ChildConstraints {
        if self.is_error_state {
            if let Some(ref error_widget) = self.error_widget {
                error_widget.get_constraints()
            } else if let Some(ref last_error) = self.last_error {
                ErrorWidget::new(last_error).get_constraints()
            } else {
                ErrorWidget::new_with_message("Error").get_constraints()
            }
        } else {
            self.widget.get_constraints()
        }
    }

    fn update(&mut self) -> Result<(), AppError> {
        if self.is_error_state {
            if let Some(ref mut error_widget) = self.error_widget {
                error_widget.update()
            } else if let Some(ref last_error) = self.last_error {
                ErrorWidget::new(last_error).update()
            } else {
                Ok(())
            }
        } else {
            // Try to update the main widget
            match self.widget.update() {
                Ok(()) => Ok(()),
                Err(e) => {
                    // Switch to error state
                    self.is_error_state = true;
                    self.last_error = Some(e);
                    if let Some(ref mut error_widget) = self.error_widget {
                        error_widget.update()
                    } else {
                        Ok(())
                    }
                }
            }
        }
    }
}