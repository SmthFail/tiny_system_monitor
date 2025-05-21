use super::buffer::Buffer;


#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16, 
    pub height: u16,
}

pub trait Widget {
    fn render(&mut self, buf: &mut Buffer, area: Rect);

    fn get_constraints(&self) -> (Option<u16>, Option<u16>);

    fn update(&mut self);
}
