use super::widget::{Widget, Rect};
use super::buffer::{Buffer, Cell as buffCell};
use std::cell::RefCell;
use std::rc::Rc;

pub struct ProgressBar {
    pub title: String,
    pub postfix: String,
    pub symbol: char,
        current_value: Rc<RefCell<f64>>,
        total_value: Rc<RefCell<f64>>
}

impl ProgressBar {
    pub fn new(title: &str, postfix: &str, current_value: & Rc<RefCell<f64>>, total_value: & Rc<RefCell<f64>>) -> Self {
        Self {
            title: title.to_owned(), 
            postfix: postfix.to_owned(),
            symbol: '|',
            current_value: current_value.clone(),
            total_value: total_value.clone()
        }
    }
}

impl Widget for ProgressBar {
    fn render(&mut self, buff: &mut Buffer, area: Rect) {
        let progress_bar_width = area.width
            .saturating_sub(self.title.len() as u16)
            .saturating_sub(self.postfix.len() as u16)
            .saturating_sub(2); // [ and ]
       
        let progress_data = self.current_value.borrow().clone() as f64 / self.total_value.borrow().clone() as f64; 
        let progress_indicator = format!("{:>5.1}%]{}", progress_data * 100.0, self.postfix);
        let load_width = (progress_bar_width as f64 * progress_data).floor() as usize;
        let remaining_width = progress_bar_width.saturating_sub(load_width as u16).saturating_sub(progress_indicator.len() as u16);
       
        let mut current_pos = area.x;
        let title_bytes = self.title.chars().take(area.width as usize).collect::<Vec<_>>();

        for  ch in title_bytes.iter() {
            buff.set_cell(current_pos, area.y, buffCell::new(*ch));
            current_pos += 1;
        }
        
        buff.set_cell(current_pos, area.y, buffCell::new('['));
        current_pos += 1; 
            
        for _ in 0..load_width {
           buff.set_cell(current_pos, area.y, buffCell::new(self.symbol)); 
           current_pos += 1;
        }
       
        for _ in 0..remaining_width {
            buff.set_cell(current_pos, area.y, buffCell::new(' '));
            current_pos += 1;
        }
        
        for ch in progress_indicator.chars() {
            buff.set_cell(current_pos, area.y, buffCell::new(ch));
            current_pos += 1
        }
    }

    fn get_constraints(&self) -> (Option<u16>, Option<u16>) {
        (Some(1), Some(1))
    }

    fn update(&mut self) {
    }
}
