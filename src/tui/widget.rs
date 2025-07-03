use super::buffer::Buffer;
use super::app_error::AppError;

#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16, 
    pub height: u16,
}

pub trait Widget {
    fn render(&mut self, buf: &mut Buffer, area: Rect) -> Result<(), AppError>;

    fn get_constraints(&self) -> (Option<u16>, Option<u16>);

    fn update(&mut self);
}
