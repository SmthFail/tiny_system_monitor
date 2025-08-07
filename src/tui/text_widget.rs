use super::buffer::{Buffer, Cell, Color};
use super::widget::{Widget, Rect, ChildConstraints};
use super::app_error::AppError;
use std::{
    rc::Rc,
    cell::RefCell
};

pub enum TextSource {
    Static(String),
    Editable(Rc<RefCell<String>>),
}

pub struct TextWidget {
    pub text: TextSource,
    pub fg: Option<Color>,
    pub bg: Option<Color>
}

impl TextWidget {
    pub fn new_static(text: &str) -> Self {
        Self {
            text: TextSource::Static(text.to_owned()),
            fg: None,
            bg: None
        }
    }

    pub fn new_editable(text: Rc<RefCell<String>>) -> Self {
        Self {
            text: TextSource::Editable(text),
            fg: None,
            bg: None
        }
    }

    pub fn set_color(mut self, fg: Option<Color>, bg: Option<Color>) -> Self {
        self.fg = fg;
        self.bg = bg;
        self
    }
}

impl Widget for TextWidget {
    fn render(&mut self, buff: &mut Buffer, area: Rect) -> Result<(), AppError>{
        let max_width = area.width as usize;

        if max_width < 3 {
            // TODO set bg to amber when bg will be added
            return Err(AppError::warning("Overflowed!"))
        }

        let mut chars: Vec<char> = match &self.text {
            TextSource::Static(s) => s.chars().collect(),
            TextSource::Editable(s) => s.borrow().chars().collect(),
        };

        if chars.len() > max_width {
            chars.truncate(max_width - 1);
            chars.push('…')
        }

        for (i, ch) in chars.iter().enumerate() {
            let x = area.x + i as u16;
            let y = area.y;
            if x < buff.width && y < buff.height {
                let cell = Cell::new(*ch).fg(self.fg.clone()).bg(self.bg.clone());
                buff.set_cell(x, y, cell);
            }
        }


        if area.width as usize > chars.len() {
            for i in chars.len()..area.width as usize {
                let cell = Cell::new(' ').bg(self.bg.clone());
                buff.set_cell(area.x + i as u16, area.y, cell)
            }
        }

        Ok(())
    }

    fn get_constraints(&self) -> ChildConstraints {
        let width = match &self.text {
            TextSource::Static(s) => s.len(),
            TextSource::Editable(s) => s.borrow().len()
        };
        ChildConstraints{
            min_width: None,
            //max_width: Some(width as u16),
            max_width: None,
            min_height: Some(1),
            max_height: Some(1)
        }
    }

    fn update(&mut self) -> Result<(), AppError>{
        Ok(())
    }
}
