use super::buffer::Buffer;


#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16, 
    pub height: u16,
}

pub trait Widget {
    fn render(&self, buf: &mut Buffer, area: Rect);

    fn size_hint(&self) -> (u16, u16);

    fn update(&mut self);
}
