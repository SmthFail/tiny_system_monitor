use super::buffer::{Buffer, Cell};
use super::widget::{Widget, Rect};

pub struct App {
    pub buffer: Buffer,
    childrens: Vec<Box<dyn Widget>>
}

impl App {
    pub fn new(width: u16, height: u16) -> Self {
        let buffer = Buffer::new(width, height);
        App {
            buffer,
            childrens: Vec::new()
        }
    }
    
    pub fn add_child(&mut self, child: Box<dyn Widget>) {
        self.childrens.push(child);
    }
    
    pub fn update(&mut self) {
        for child in &mut self.childrens {
            child.update()
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.buffer = Buffer::new(width, height);
        // TODO update size of child widget?
    }

    pub fn render(&mut self) {
        // check numbers of fixed lines and number of flexed widget 
        let mut fixed_lines: u16 = 0;
        let mut flex_widgets: u16 = 0;

        for child in &self.childrens {
            let (_, h) = child.get_constraints();
            match h {
                Some(h) => fixed_lines += h,
                None => flex_widgets += 1,
            } 
        }

        if fixed_lines + flex_widgets >= self.buffer.height{
           for (i, char) in "Overflowed".chars().enumerate() {
               self.buffer.set_cell(i as u16, 0, Cell::new(char));
           }
           return;
        }

        let mut current_row = 0;
        for child in &mut self.childrens {
            let (_, h) = child.get_constraints();
            let child_area = match h {
                Some(h) => Rect {
                    x: 0,
                    y: current_row,
                    width: self.buffer.width,
                    height: h
                },
                None => Rect {
                    x: 0,
                    y: current_row,
                    width: self.buffer.width,
                    height: (self.buffer.height - fixed_lines / flex_widgets)
                }
            };
            child.render(
                &mut self.buffer,
                child_area
                );
            current_row += child_area.height;
        }
    }
}
