use super::buffer::{Buffer, Cell, Color};
use super::{
    widget::{Widget, Rect},
    app_error::AppError
};
use std::cell::RefCell;
use std::rc::Rc;

pub struct EditableTextWidget {
    pub text: Rc<RefCell<String>>,
    pub fg: Option<Color>,
}

impl EditableTextWidget {
    pub fn new(text: &Rc<RefCell<String>>) -> Self {
        Self {
            text: text.clone(),
            fg: None,
        }
    }
}

impl Widget for EditableTextWidget {
    fn render(&mut self, buff: &mut Buffer, area: Rect) -> Result<(), AppError>{
        let max_width = area.width as usize;
        let text_bytes = self.text.borrow_mut().chars().take(max_width).collect::<Vec<_>>();

        for (i, ch) in text_bytes.iter().enumerate() {
            let x = area.x + i as u16;
            let y = area.y;
            if x < buff.width && y < buff.height {
                let cell = Cell::new(*ch);
                buff.set_cell(x, y, cell);
            }
        }

        // clear rest of line
        let usize_width = area.width as usize;
        if usize_width > text_bytes.len() {
            for i in text_bytes.len()..usize_width - 1 { // TODO fix -1 with correct send area from
                                                         // container. Not include border now
                let cell = Cell::new(' ');
                buff.set_cell(area.x + i as u16, area.y, cell)
            }
        }
        Ok(())
    }

    fn get_constraints(&self) -> (Option<u16>, Option<u16>) {
        (Some(self.text.borrow_mut().len() as u16), Some(1))
    }

    fn update(&mut self) {

    }
}
