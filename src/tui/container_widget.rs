use super::widget::{Widget, Rect};
use super::buffer::{Buffer, Cell};


pub enum Layout {
    Vertical,
    Horizontal,
    Grid { rows: u16, columns: u16 },
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
        border: bool
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

    fn _draw_vertical(&mut self, inner: Rect, buff: &mut Buffer) {
        let spacer_height = match self.alignment {
            Alignment::Start => {
               0 
            },
            Alignment::Center => {
                1
            }
            Alignment::End => {
                2
            }
        };
        

        let mut current_row = inner.y;
        let children_count = self.children.len();
        let mut total_height = 0;
        
        for (i, child) in &mut self.children.iter_mut().enumerate() {
            let desire_height = if i == children_count - 1 {
                inner.height - total_height
            } else {
                inner.height / children_count as u16
            };

            let (_, h) = child.get_constraints();
            let child_rect = match h {
                Some(h) => Rect {
                   x: inner.x,
                   y: current_row,
                   width: inner.width,
                   height: h + spacer_height 
                },
                None => Rect {
                    x: inner.x,
                    y: current_row,
                    width: inner.width,
                    height: desire_height 
                }
             };

               child.render(buff, child_rect);
               current_row += child_rect.height;
               total_height += desire_height
       }
    }
}

impl Widget for Container {

    fn render(&mut self, buff: &mut Buffer, area: Rect) {
       let border_size = if self.border {
          1 
       } else {
          0
       };

       if self.border {
           self.draw_border(buff, area)
       }
        
       // check that it at least 1 row inside widget
       if area.width <= 1 + border_size {
           return
       } 
       
       // inner area for children
       let inner = Rect {
           x: area.x + border_size,
           y: area.y + border_size,
           width: area.width - border_size,
           height: area.height - border_size
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
              self._draw_vertical(inner, buff);
           }
           Layout::Horizontal => {
                let child_width = inner.width / num_children as u16;
                let mut current_col = inner.x;
                for (i, child) in self.children.iter_mut().enumerate() {
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

    fn get_constraints(&self) -> (Option<u16>, Option<u16>) {
        (self.width, self.height)
    }

    fn update(&mut self) {
        for child in &mut self.children {
            child.update();
        }
    }
}
