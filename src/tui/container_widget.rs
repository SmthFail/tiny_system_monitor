use super::{
    widget::{Widget, Rect, ChildConstraints},
    buffer::{Buffer, Cell},
    app_error::AppError
};


pub enum Layout {
    Vertical,
    Horizontal,
    Grid,  
}

pub enum Alignment {
    Start,
    Center,
    End
}

pub struct Container {
    pub width: Option<u16>,
    pub height: Option<u16>,
    pub children: Vec<Box<dyn Widget>>,
    pub layout: Layout,
    pub alignment: Alignment,
        border: bool,
}

impl Container {
    pub fn new(
        width: Option<u16>, 
        height: Option<u16>, 
        layout: Layout, 
        alignment: Alignment,
        border: bool) -> Self {
        Self {
            children: Vec::new(),
            width,
            height,
            layout,
            alignment,
            border
        }
    }

    pub fn add_child(&mut self, w: Box<dyn Widget>) {
        self.children.push(w);
    }

    fn draw_border(&self, buff: &mut Buffer, area: Rect) {
       let x1 = area.x;
       let y1 = area.y;
       let x2 = x1 + area.width.saturating_sub(1);
       let y2 = y1 + area.height.saturating_sub(1);
       
       //draw corners
       buff.set_cell(x1, y1, Cell::new('┌'));
       buff.set_cell(x2, y1, Cell::new('┐'));
       buff.set_cell(x1, y2, Cell::new('└'));
       buff.set_cell(x2, y2, Cell::new('┘'));
       
       // top and bottom border
       for x in (x1 + 1)..x2 {
           if y1 < buff.height {
               buff.set_cell(x, y1, Cell::new('─'));
           }
           if y2 < buff.height && y2 > y1 {
               buff.set_cell(x, y2, Cell::new('─'));
           }
       }

       // left and right border
       for y in (y1 + 1)..y2 {
            if x1 < buff.width {
                buff.set_cell(x1, y, Cell::new('│'));
            }
            if x2 < buff.width && x2 > x1 {
                buff.set_cell(x2, y, Cell::new('│'));
            }
       }
    }

    fn make_columns(heights: &[u16], max_height: u16) -> Vec<Vec<usize>> {
        let mut columns: Vec<Vec<usize>> = vec![Vec::new()];
        let mut current_height: u16 = 0;

        for (idx, &h) in heights.iter().enumerate() {
            // TODO check if child height > max_height and return overflowed
            if !columns.last().unwrap().is_empty() && current_height + h > max_height {
                columns.push(Vec::new());
                current_height = 0;
            }

            columns.last_mut().unwrap().push(idx);
            current_height += h;
        }
        columns
    }

    fn _draw_grid(&mut self, inner: Rect, buff: &mut Buffer) -> Result<(), AppError>{
        let child_sizes = match self.calculate_childs_size(inner) {
            Some(size) => size,
            None => return Ok(())
        };
        
        let heights: Vec<u16> = child_sizes.iter().map(|&(_, h)| h).collect();

        let columns = Self::make_columns(&heights, inner.height);

        let n_cols = columns.len() as u16;
        let base_w = inner.width / n_cols;
        let extra = inner.width % n_cols;

        if base_w == 0 {
            return Err(AppError::warning("Base w is 0"));
        }

        let mut x = inner.x;
        for (i, col) in columns.into_iter().enumerate() {
            let w = base_w + if (i as u16) < extra {1} else {0};
            let mut y = inner.y;
            for idx in col {
                let h = child_sizes[idx].1;
                let rect = Rect {x, y, width: w, height: h};
                self.children[idx].render(buff, rect)?;
                y += h;
            }
            x += w;
        }
        Ok(())
        
    }


    fn _draw_vertical(&mut self, inner: Rect, buff: &mut Buffer, childs_size: Vec<(u16, u16)>) -> Result<(), AppError>{
        let mut current_height = 0;
        let mut is_child_error: Result<(), AppError> = Ok(());
        for (i, child) in self.children.iter_mut().enumerate() {
            if current_height + childs_size[i].1 > inner.height {
                is_child_error = Err(AppError::warning("Overflowed"));
                continue;
            }
            let child_rect = Rect {
                x: inner.x,
                y: inner.y + current_height,
                width: inner.width,
                height: childs_size[i].1
            };
            child.render(buff, child_rect)
                .unwrap_or_else(|e| is_child_error = Err(e));
            current_height += child_rect.height;
        }
        is_child_error 
    }

    fn _draw_horizontal(&mut self, inner: Rect, buff: &mut Buffer, childs_size: Vec<(u16, u16)>) -> Result<(), AppError>{
        let mut current_col = inner.x;
        let mut is_child_error: Result<(), AppError> = Ok(());
        for (i, child) in self.children.iter_mut().enumerate() {
            let child_rect = Rect {
                x: current_col,
                y: inner.y,
                width: childs_size[i].0,
                height: inner.height
            };
            child.render(buff, child_rect)
                .unwrap_or_else(|e| is_child_error = Err(e));
            current_col += child_rect.width;
        }
        is_child_error
    }


    fn calculate_childs_size(&mut self, area: Rect) -> Option<Vec<(u16, u16)>>{
       if self.children.is_empty() {
           return None;
       } 
       
       let mut flex_vertical_count = 0;
       let mut flex_horizontal_count = 0;
       let mut fixed_vertical_size = 0;
       let mut fixed_horizontal_size = 0;
        

       let mut childs_size: Vec<(u16, u16)> = self.children.iter().map(|child|{
           let constraints = child.get_constraints();
          
           // TODO correct check
           let w = constraints.max_width.unwrap_or(0);
           let h = constraints.max_height.unwrap_or(0);

           if w == 0 {
              flex_horizontal_count += 1; 
           } else {
              fixed_horizontal_size += w;
           }

           if h == 0 {
               flex_vertical_count += 1;
           } else {
               fixed_vertical_size += h;
           }

           (w, h)
       }).collect(); 

       let flex_h_size = if flex_horizontal_count > 0 {
           let remaining_space = area.width.saturating_sub(fixed_horizontal_size);
           remaining_space / flex_horizontal_count
       } else {
           0
       };

       let flex_v_size = if flex_vertical_count > 0 {
           let remaining_space = area.height.saturating_sub(fixed_vertical_size);
           remaining_space / flex_vertical_count 
       } else {
           0
       };

       for (w, h) in childs_size.iter_mut() {
           if *w == 0 {
               *w = flex_h_size
           }
           if *h == 0 {
               *h = flex_v_size
           }
       }

       Some(childs_size)
    }


}

impl Widget for Container {

     fn render(&mut self, buff: &mut Buffer, area: Rect) -> Result<(), AppError>{
        // Check for at least 1 column and row exist 
        if self.border {
            if area.width < 3 || area.height < 3 {
                return Err(AppError::warning("Overflowed"))
            }
        } else {
            if area.width < 1 || area.height < 1 {
                return Err(AppError::warning("Overflowed"))
            }
        }

        // Calculate draw rectangle for childs. Draw border if needed
        let inner = if self.border {
            self.draw_border(buff, area);
            Rect {
                x: area.x + 1,
                y: area.y + 1,
                width: area.width - 2,
                height: area.height - 2
            }
        } else {
            Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: area.height
            }
        };
        
        // Draw children
        if self.children.len() == 0 {
           return Ok(());
        }

        let childs_size = match self.calculate_childs_size(inner) {
           Some(size) => size,
           None => return Ok(())
        };

        match self.layout {
           Layout::Vertical => {
              self._draw_vertical(inner, buff, childs_size)?;
           }
           Layout::Horizontal => {
              self._draw_horizontal(inner, buff, childs_size)?;
           }
           Layout::Grid => {
              self._draw_grid(inner, buff)?;
           }
       }
      
       Ok(())
    }

    fn get_constraints(&self) -> ChildConstraints {
        ChildConstraints {
            min_width: self.width,
            max_width: self.width,
            min_height: self.height,
            max_height: self.height 
        }
    }

    fn update(&mut self) -> Result<(), AppError>{
        for child in &mut self.children {
            child.update()?;
        }
        Ok(())
    }
}
