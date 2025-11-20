use super::buffer::{Buffer, Cell, Color};
use super::widget::{Widget, Rect, ChildConstraints};
use super::app_error::AppError;
use crate::tui::cell_types::CellString;

// Helper function to wrap text to fit within specified width
fn wrap_text(text: &str, max_width: usize, max_lines: usize) -> Vec<String> {
    let mut lines = Vec::new();

    // If max_width is too small to be useful, return the text as a single line
    if max_width < 3 {
        return vec![text.to_string()];
    }

    // First, split by actual newlines in the text
    for paragraph in text.split('\n') {
        // Then wrap each paragraph by max_width
        let wrapped_paragraph_lines = wrap_line(paragraph, max_width);
        lines.extend(wrapped_paragraph_lines);

        // If we've hit the maximum number of lines, return early
        if lines.len() >= max_lines {
            break;
        }
    }

    // Only keep up to max_lines
    if lines.len() > max_lines {
        lines.truncate(max_lines);
    }

    lines
}

// Helper function to wrap a single line of text by max_width
fn wrap_line(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let words: Vec<&str> = text.split_whitespace().collect();

    if words.is_empty() {
        if !text.is_empty() {
            // If text contains only whitespace or special characters, handle it appropriately
            lines.push(text.to_string());
        } else {
            // Empty text
            lines.push(String::new());
        }
        return lines;
    }

    let mut current_line = String::new();

    for word in words {
        // Create a test line to see if the word fits
        let test_line = if current_line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", current_line, word)
        };

        if test_line.len() <= max_width {
            // Word fits in the current line
            current_line = test_line;
        } else {
            // Word doesn't fit, so we need to start a new line

            // If we have content in current line, add it first
            if !current_line.is_empty() {
                lines.push(current_line);
            }

            // Now handle the word that didn't fit - might need to break it
            if word.len() > max_width {
                // Word is longer than max_width, break it into chunks
                let word_chars: Vec<char> = word.chars().collect();
                let chunks: Vec<String> = word_chars
                    .chunks(max_width.saturating_sub(1)) // -1 to potentially allow for continuation chars
                    .map(|chunk| chunk.iter().collect())
                    .collect();

                // Add all but the last chunk as full lines
                for chunk in chunks.iter().take(chunks.len() - 1) {
                    lines.push(chunk.clone());
                }

                // The last chunk becomes the new current line
                current_line = chunks.last().unwrap_or(&String::new()).clone();
            } else {
                // Word is short enough to be its own line
                current_line = word.to_string();
            }
        }
    }

    // Add the last line if it's not empty
    if !current_line.is_empty() {
        lines.push(current_line);
    }

    // If the text was empty, add an empty line
    if lines.is_empty() && !text.is_empty() {
        lines.push(text.to_string());
    }

    lines
}

pub enum TextSource {
    Static(String),
    Editable(CellString),
}

pub struct TextWidget {
    pub text: TextSource,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    max_lines: Option<u16>,
}

impl TextWidget {
    pub fn new_static(text: &str) -> Self {
        Self {
            text: TextSource::Static(text.to_owned()),
            fg: None,
            bg: None,
            max_lines: None,
        }
    }

    pub fn new_editable(text: CellString) -> Self {
        Self {
            text: TextSource::Editable(text),
            fg: None,
            bg: None,
            max_lines: None,
        }
    }

    pub fn set_color(mut self, fg: Option<Color>, bg: Option<Color>) -> Self {
        self.fg = fg;
        self.bg = bg;
        self
    }

    pub fn max_lines(mut self, max_lines: u16) -> Self {
        self.max_lines = Some(max_lines);
        self
    }
}

impl Widget for TextWidget {
    fn render(&mut self, buff: &mut Buffer, area: Rect) -> Result<(), AppError>{
        let max_width = area.width as usize;
        let max_height = area.height as usize;

        if max_width < 3 {
            // TODO set bg to amber when bg will be added
            return Err(AppError::warning("Overflowed!"))
        }

        // Handle the borrowing issue by getting the string content inside the match
        let text_content = match &self.text {
            TextSource::Static(s) => s.clone(), // Get owned string
            TextSource::Editable(s) => s.get_str(), // Get owned string
        };

        // Determine the maximum number of lines to display
        let max_lines = self.max_lines.unwrap_or(1) as usize;
        let effective_max_lines = std::cmp::min(max_lines, max_height);

        // Split text into lines
        let text_lines = wrap_text(&text_content, max_width, effective_max_lines);

        // If there are more lines than allowed by max_lines, we need to indicate overflow
        let has_overflow = text_lines.len() > effective_max_lines;

        // Render each line
        for (line_idx, line) in text_lines.iter().take(effective_max_lines).enumerate() {
            let y = area.y + line_idx as u16;

            if y >= buff.height {
                break; // Don't render outside the buffer
            }

            let line_chars: Vec<char> = line.chars().collect();
            let mut truncated_line = line_chars;

            // Check if the current line needs to be truncated
            let is_last_shown_line = line_idx == effective_max_lines - 1;

            if truncated_line.len() > max_width {
                if is_last_shown_line && has_overflow {
                    // If this is the last line we're showing and there are more lines
                    // coming after, add "...more" indicator (3 chars)
                    if max_width >= 3 {
                        truncated_line.truncate(max_width - 3);
                        truncated_line.extend_from_slice(&['.', '.', '.']);
                    } else {
                        // If max_width is too small to show "..."
                        truncated_line.truncate(max_width);
                    }
                } else if truncated_line.len() > max_width {
                    // If line is too long but not the overflow case, truncate with "…"
                    if max_width >= 1 {
                        truncated_line.truncate(max_width - 1);
                        truncated_line.push('…');
                    } else {
                        truncated_line.truncate(max_width);
                    }
                }
            }

            // Render characters of this line
            for (char_idx, ch) in truncated_line.iter().enumerate() {
                let x = area.x + char_idx as u16;
                if x < buff.width {
                    let cell = Cell::new(*ch).fg(self.fg.clone()).bg(self.bg.clone());
                    buff.set_cell(x, y, cell);
                }
            }

            // Fill remaining space on line with background color
            for char_idx in truncated_line.len()..max_width {
                let x = area.x + char_idx as u16;
                if x < buff.width {
                    let cell = Cell::new(' ').bg(self.bg.clone());
                    buff.set_cell(x, y, cell);
                }
            }
        }

        Ok(())
    }

    fn get_constraints(&self) -> ChildConstraints {
        let _text_content = match &self.text {
            TextSource::Static(s) => s.clone(), // Get owned string
            TextSource::Editable(s) => s.get_str(), // Get owned string
        };

        let has_multiline = self.max_lines.is_some() && self.max_lines.unwrap() > 1;

        let mut constraints = ChildConstraints {
            min_width: Some(1),
            max_width: None,
            min_height: Some(1),
            max_height: if has_multiline { self.max_lines } else { Some(1) },
        };

        // If we have a max_lines constraint, set max_height accordingly
        if let Some(max_lines) = self.max_lines {
            constraints.max_height = Some(max_lines);
        }

        constraints
    }

    fn update(&mut self) -> Result<(), AppError>{
        Ok(())
    }
}
