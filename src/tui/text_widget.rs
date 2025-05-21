use super::buffer::{Buffer, Cell, Color};
use super::widget::{Widget, Rect};

pub struct TextWidget {
    pub text: String,
    pub fg: Option<Color>,
}

impl TextWidget {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            fg: None,
        }
    }
}

impl Widget for TextWidget {
    fn render(&mut self, buff: &mut Buffer, area: Rect) {
        let max_width = area.width as usize;
        let text_bytes = self.text.chars().take(max_width).collect::<Vec<_>>();

        for (i, ch) in text_bytes.iter().enumerate() {
            let x = area.x + i as u16;
            let y = area.y;
            if x < buff.width && y < buff.height {
                let cell = Cell::new(*ch);
                buff.set_cell(x, y, cell);
            }
        }
    }

    fn get_constraints(&self) -> (Option<u16>, Option<u16>) {
        (Some(self.text.len() as u16), Some(1))
    }

    fn update(&mut self) {

    }
}
