use super::widget::Widget;
use super::buffer::{Buffer, Cell};
use super::widget::Rect;


pub enum Layout {
    Vertical,
    Horizontal,
    Grid { rows: u16, columns: u16 },
}

pub struct Tile {
    pub width: u16,
    pub height: u16,
    pub children: Vec<Box<dyn Widget>>,
    pub layout: Layout,
}

impl Tile {
    pub fn new(width: u16, height: u16, layout: Layout) -> Self {
        Self {
            children: Vec::new(),
            width,
            height,
            layout,
        }
    }

    pub fn add_child(&mut self, w: Box<dyn Widget>) {
        self.children.push(w);
    }
}

impl Widget for Tile {
    fn render(&self, buff: &mut Buffer, area: Rect) {
       let x1 = area.x;
       let y1 = area.y;
       let x2 = x1 + area.width.saturating_sub(1);
       let y2 = y1 + area.height.saturating_sub(1);

       // top and bottom border
       for x in x1..=x2 {
           if y1 < buff.height {
               buff.set_cell(x, y1, Cell::new('#'));
           }
           if y2 < buff.height && y2 > y1 {
               buff.set_cell(x, y2, Cell::new('#'));
           }
       }

       // left and right border
       for y in y1..=y2 {
            if x1 < buff.width {
                buff.set_cell(x1, y, Cell::new('#'));
            }
            if x2 < buff.width && x2 > x1 {
                buff.set_cell(x2, y, Cell::new('#'));
            }
       }

       // inner area for children
       let inner = Rect {
           x: x1 + 1,
           y: y1 + 1,
           width: area.width.saturating_sub(2),
           height: area.height.saturating_sub(2)
       };

       if inner.width == 0 || inner.height == 0 {
           return;
       }

       // draw children
       let num_children = self.children.len();
       if num_children == 0 {
           return;
       }

       match self.layout {
           Layout::Vertical => {
               let child_height = inner.height / num_children as u16;
               let mut current_row = inner.y;

               for (i, child) in self.children.iter().enumerate() {
                   let heigth_for_child= if i == num_children - 1 {
                       inner.y + inner.height - current_row
                   } else {
                       child_height
                   };

                   let child_rect = Rect {
                       x: inner.x,
                       y: current_row,
                       width: inner.width,
                       height: heigth_for_child
                   };

                   child.render(buff, child_rect);
                   current_row += heigth_for_child;
               }
           }
           Layout::Horizontal => {
                let child_width = inner.width / num_children as u16;
                let mut current_col = inner.x;
                for (i, child) in self.children.iter().enumerate() {
                    let width_for_child = if i == num_children -1 {
                        inner.x + inner.width - current_col
                    } else {
                        child_width
                    }; 
                

                    let child_rect = Rect {
                        x: current_col,
                        y: inner.y,
                        width: width_for_child,
                        height: inner.height
                    };
                    
                    child.render(buff, child_rect);
                    current_col += width_for_child;
                }
           }
           Layout::Grid { rows, columns } => {
                let cell_width = inner.width / columns;
                let cell_heigth = inner.height / rows;
                let mut child_index = 0;

                for row in 0..rows {
                    for col in 0..columns {
                        if child_index >= num_children {
                            break;
                        }

                        let child_rect = Rect {
                            x: inner.x + col * cell_width,
                            y: inner.y + row * cell_heigth,
                            width: cell_width,
                            height: cell_heigth
                        };

                        self.children[child_index].render(buff, child_rect);
                        child_index += 1
                    }
                }
           }
       }
       
    }

    fn size_hint(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    fn update(&mut self) {
        for child in &mut self.children {
            child.update();
        }
    }
}
