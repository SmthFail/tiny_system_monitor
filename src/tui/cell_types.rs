use std::rc::Rc;
use std::cell::RefCell;

#[derive(Clone)]
pub struct CellString {
    data: Rc<RefCell<String>>
}

impl CellString {
    pub fn new() -> Self {
        CellString {data: Rc::new(RefCell::new(String::new()))}

    }

    pub fn update(&mut self, value: String)  {
        *self.data.borrow_mut() = value;
    }

    pub fn clear(&mut self) {
        *self.data.borrow_mut() = String::new();
    }

    pub fn get_str(&self) -> String{
        self.data.borrow_mut().clone()
    }
}

