use super::widget::{Widget, Rect};
use super::{
    buffer::{Buffer, Cell as buffCell, Color},
    app_error::AppError
};
use std::cell::RefCell;
use std::rc::Rc;

pub struct ProgressBar {
    pub title: String,
    pub postfix: String,
    pub symbol: char,
        current_value: Rc<RefCell<f64>>,
        total_value: Rc<RefCell<f64>>,
        use_percent: bool
}

impl ProgressBar {
    pub fn new(title: &str, postfix: &str, current_value: & Rc<RefCell<f64>>, total_value: & Rc<RefCell<f64>>, use_percent: bool) -> Self {
        Self {
            title: title.to_owned(), 
            postfix: postfix.to_owned(),
            symbol: '|',
            current_value: current_value.clone(),
            total_value: total_value.clone(),
            use_percent
        }
    }
}

impl Widget for ProgressBar {
    fn render(&mut self, buff: &mut Buffer, area: Rect) -> Result<(), AppError>{
        // process data 
        let total = (*self.total_value.borrow()).max(1.0);
        let progress = (*self.current_value.borrow() / total).clamp(0.0, 1.0);

        //format indicator

        let indicator: String = if self.use_percent {
            format!("{:>5.1}{}", progress * 100.0, self.postfix)
        }
        else {
            format!("{:.1}/{:.1}{}", self.current_value.borrow(), self.total_value.borrow() , self.postfix)
        };

        // calculate min width: title + '[' + bar + ']' + indicator
        let min_width = self.title.len() as u16 + indicator.len() as u16 + 3; // TODO check wh 3?
                                                                              
                                                                              
        if area.width < min_width {
            for (i, ch) in self.title.chars().take(area.width as usize).enumerate() {
                buff.set_cell(area.x + i as u16, area.y, buffCell::new(ch));
            }
            return Err(AppError::warning("Overflowed"));
        } 


        let bar_width = area.width - min_width;
        let filled = (bar_width as f64 * progress).floor() as u16;
        let empty = bar_width - filled; 
      

        // render bar
        let mut current_pos = area.x;
        let title_bytes = self.title.chars().take(area.width as usize).collect::<Vec<_>>();

        for ch in title_bytes.iter() {
            buff.set_cell(current_pos, area.y, buffCell::new(*ch));
            current_pos += 1;
        }
        
        buff.set_cell(current_pos, area.y, buffCell::new('['));
        current_pos += 1; 
            
        // filled part 
        
        let bar_color = match progress  {
            0.0..=0.5  => Color::Green,
            0.5..=0.75 => Color::Yellow,
            _       => Color::Red,
        };

        let bar_symbol = buffCell::new(self.symbol).fg(Some(bar_color));
        for _ in 0..filled {
           buff.set_cell(current_pos, area.y, bar_symbol.clone()); 
           current_pos += 1;
        }
       
        for _ in 0..empty {
            buff.set_cell(current_pos, area.y, buffCell::new(' '));
            current_pos += 1;
        }
        
        for ch in indicator.chars() {
            buff.set_cell(current_pos, area.y, buffCell::new(ch));
            current_pos += 1
        }

        buff.set_cell(current_pos, area.y, buffCell::new(']'));

        Ok(())
    }

    fn get_constraints(&self) -> (Option<u16>, Option<u16>) {
        (Some(1), Some(1))
    }

    fn update(&mut self) -> Result<(), AppError> {
        Ok(())
    }
}
