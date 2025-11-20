use crate::tui::buffer::{Buffer, Cell, Color};
use crate::tui::widget::{Widget, Rect, ChildConstraints};
use crate::tui::app_error::AppError;

pub struct ErrorWidget {
    error_message: String,
}

impl ErrorWidget {
    pub fn new(error: &AppError) -> Self {
        Self::new_with_message(&error.message)
    }

    pub fn new_with_message(message: &str) -> Self {
        Self {
            error_message: message.to_string(),
        }
    }
}

impl Widget for ErrorWidget {
    fn render(&mut self, buf: &mut Buffer, area: Rect) -> Result<(), AppError> {
        // Fill the entire area with black background
        for y in area.y..(area.y + area.height) {
            for x in area.x..(area.x + area.width) {
                if x < buf.width && y < buf.height {
                    let cell = Cell::new(' ').bg(Some(Color::Black));
                    buf.set_cell(x, y, cell);
                }
            }
        }

        // Calculate centered position for error text
        let text_chars: Vec<char> = self.error_message.chars().collect();
        let text_len = text_chars.len();
        
        if text_len > 0 && area.width > 0 && area.height > 0 {
            // Truncate text if it's too long for the area
            let max_text_len = area.width as usize;
            let display_chars = if text_len > max_text_len {
                let mut truncated: Vec<char> = text_chars[..max_text_len-1].to_vec();
                truncated.push('…'); // Add ellipsis
                truncated
            } else {
                text_chars
            };
            
            let display_len = display_chars.len();
            let start_x = area.x + (area.width.saturating_sub(display_len as u16)) / 2;
            let start_y = area.y + area.height / 2;

            // Draw the error text centered vertically and horizontally
            for (i, ch) in display_chars.iter().enumerate() {
                let x = start_x + i as u16;
                let y = start_y;
                
                if x < buf.width && y < buf.height && x < area.x + area.width {
                    let cell = Cell::new(*ch).fg(Some(Color::Red)).bg(Some(Color::Black));
                    buf.set_cell(x, y, cell);
                }
            }
        }

        Ok(())
    }

    fn get_constraints(&self) -> ChildConstraints {
        ChildConstraints {
            min_width: Some(5),  // Minimum width to show a short error message
            max_width: None,
            min_height: Some(1), // Minimum height for basic visibility
            max_height: None,
        }
    }
}