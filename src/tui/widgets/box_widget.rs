use crate::{
    Widget,
    Buffer,
};
use crate::tui::widget::{Rect, ChildConstraints};
use crate::tui::app_error::AppError;
use crate::tui::buffer::Cell;




pub struct BoxWidget {
    border: bool,
    name: String,
    child: Option<Box<dyn Widget>>
}

impl BoxWidget {
    pub fn new() -> Self {
        Self{border: false, name: String::new(), child: None}
    }

    pub fn border(mut self, border: bool) -> Self {
        self.border = border;
        self 
    }

    pub fn child(mut self, child: impl Widget + 'static) -> Self {
        self.child = Some(Box::new(child));
        self
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    fn draw_border(&self, buff: &mut Buffer, area: Rect) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let x1 = area.x;
        let y1 = area.y;
        let x2 = x1 + area.width.saturating_sub(1);
        let y2 = y1 + area.height.saturating_sub(1);

        // corners
        buff.set_cell(x1, y1, Cell::new('┌'));
        buff.set_cell(x2, y1, Cell::new('┐'));
        buff.set_cell(x1, y2, Cell::new('└'));
        buff.set_cell(x2, y2, Cell::new('┘'));

        
        // generate top border with name
        
        let mut top_border = vec!['─'; area.width.into()];
        if !self.name.is_empty() {
            let mut chars: Vec<char> = self.name.chars().collect();
            // we wanna truncate name and leave 1 symbol before and after it
            if chars.len() > (area.width - 2).into() {
                chars.truncate(area.width as usize - 3);
                chars.push('…')
            }
            for i in 0..chars.len() {
                top_border[i] = chars[i]
            }
        };

        // top and bottom
        for (ind, x) in ((x1 + 1)..x2).enumerate() {
            // draw name
            buff.set_cell(x, y1, Cell::new(top_border[ind]));
        }

        // bottom
        for x in (x1 + 1)..x2 {
          buff.set_cell(x, y2, Cell::new('─'));
        }

        // left and right
        for y in (y1 + 1)..y2 {
            buff.set_cell(x1, y, Cell::new('│'));
            if x2 > x1 {
                buff.set_cell(x2, y, Cell::new('│'));
            }
        }
    }
}

impl Widget for BoxWidget {
    fn render(&mut self, buff: &mut Buffer, area: Rect) -> Result<(), AppError> {
        
        let child_area =if self.border {
            self.draw_border(buff, area);
            Rect {
                x: area.x + 1,
                y: area.y + 1,
                width: area.width.saturating_sub(2),
                height: area.height.saturating_sub(2),
            }
        } else {
            area
        };

        match self.child.as_mut() {
            Some(child) => child.render(buff, child_area),
            None => Ok(())
        }

    }

    fn get_constraints(&self) -> ChildConstraints {
        let mut box_constraints = if self.border {
             ChildConstraints {
                min_width: Some(2),
                min_height: Some(2),
                max_width: None,
                max_height: None
            }

        } else {
            ChildConstraints {
                min_width: None,
                min_height: None,
                max_width: None,
                max_height: None,
            }

        };

        match &self.child {
            Some(child) => {
                let child_constraints = child.get_constraints();
                box_constraints += child_constraints;
                box_constraints

            },
            None => box_constraints 
        }
    }

    fn update(&mut self) -> Result<(), AppError> {
        match self.child.as_mut() {
            Some(child) => child.update(),
            None => Ok(())
        }
    }
}
