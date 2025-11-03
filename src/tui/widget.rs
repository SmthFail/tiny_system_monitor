use super::buffer::Buffer;
use super::app_error::AppError;
use std::ops::{AddAssign, Add};
use std::any::type_name;


pub struct WidgetBox(pub Box<dyn Widget>);

// любой конкретный виджет
impl<W: Widget + 'static> From<W> for WidgetBox {
    fn from(w: W) -> Self { WidgetBox(Box::new(w)) }
}

// (необязательно, но удобно) уже готовый trait-объект
impl From<Box<dyn Widget>> for WidgetBox {
    fn from(w: Box<dyn Widget>) -> Self { WidgetBox(w) }
}

impl WidgetBox {
    // утилита на случай, если где-то остался Box<Concrete>
    pub fn from_box<W: Widget + 'static>(w: Box<W>) -> Self {
        WidgetBox(w as Box<dyn Widget>)
    }
}


#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16, 
    pub height: u16,
}

#[derive(Clone, Default, Copy, Debug)]
pub struct ChildConstraints {
    pub min_width: Option<u16>,
    pub max_width: Option<u16>,
    pub min_height: Option<u16>,
    pub max_height: Option<u16>
}

fn add_opt(v: Option<u16>, rhs: u16) -> Option<u16> {
    v.map(|x| x.saturating_add(rhs))
}

impl Add<u16> for ChildConstraints {
    type Output = Self;

    fn add(self, rhs: u16) -> Self {
        Self {
            min_width: add_opt(self.min_width, rhs),
            max_width:  add_opt(self.max_width,  rhs),
            min_height: add_opt(self.min_height, rhs),
            max_height: add_opt(self.max_height, rhs),
        }
    }
}

impl AddAssign<u16> for ChildConstraints {
    fn add_assign(&mut self, rhs: u16) {
        *self = *self + rhs;
    }
}

impl AddAssign for ChildConstraints {
    

    fn add_assign(&mut self, rhs: Self) {
        fn add(a: Option<u16>, b: Option<u16>) -> Option<u16> {
            match (a, b) {
                (Some(x), Some(y)) => Some(x.saturating_add(y)),
                (Some(x), None) => Some(x),
                (None, Some(y)) => Some(y),
                (None, None) => None
            }
        }

        self.min_width = add(self.min_width, rhs.min_width);
        self.max_width = add(self.max_width, rhs.max_width);
        self.min_height = add(self.min_height, rhs.min_height);
        self.max_height = add(self.max_height, rhs.max_height);
    }
}


fn type_name_short<T: ?Sized>() -> &'static str {
    type_name::<T>().rsplit("::").next().unwrap_or("Widget")
}

pub trait Widget {
    fn render(&mut self, buf: &mut Buffer, area: Rect) -> Result<(), AppError>;

    fn get_constraints(&self) -> ChildConstraints;

    fn update(&mut self) -> Result<(), AppError>;

    fn get_children(&self) -> &[Box<dyn Widget>] {&[]}

    fn display_name(&self) -> &'static str {
        type_name_short::<Self>()
    }
}
