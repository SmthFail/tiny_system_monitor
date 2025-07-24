use super::buffer::Buffer;
use super::app_error::AppError;

#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16, 
    pub height: u16,
}

#[derive(Clone)]
pub struct ChildConstraints {
    pub min_width: Option<u16>,
    pub max_width: Option<u16>,
    pub min_height: Option<u16>,
    pub max_height: Option<u16>
}

pub trait Widget {
    fn render(&mut self, buf: &mut Buffer, area: Rect) -> Result<(), AppError>;

    fn get_constraints(&self) -> ChildConstraints;

    fn update(&mut self) -> Result<(), AppError>;
}
