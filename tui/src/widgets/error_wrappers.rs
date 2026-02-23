use crate::widget::Widget;
use crate::widgets::error_handling_widget::ErrorHandlingWidget;
use crate::widgets::error_widget::ErrorWidget;

/// Creates an error-handling wrapper around any widget
pub fn with_error_handling<W: Widget + 'static>(widget: W) -> impl Widget {
    ErrorHandlingWidget::new(widget)
}

/// Creates an error widget directly from an error message
pub fn error_widget_from_message(message: &str) -> impl Widget {
    ErrorWidget::new_with_message(message)
}

/// Creates an error widget directly from an AppError
pub fn error_widget_from_error<E: Into<crate::app_error::AppError>>(error: E) -> impl Widget {
    ErrorWidget::new(&error.into())
}
