use crate::app_error::AppError;
use crate::buffer::Buffer;

use super::tree::{Element, ElementKind};

pub fn paint(buf: &mut Buffer, e: &mut Element) -> Result<(), AppError> {
    match &mut e.kind {
        ElementKind::Leaf(cell) => cell.borrow_mut().render(buf, e.rect),
        ElementKind::Container { .. } => {
            for ch in &mut e.children {
                paint(buf, ch)?;
            }
            Ok(())
        }
    }
}
