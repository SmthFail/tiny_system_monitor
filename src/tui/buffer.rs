
#[derive(Clone, Debug, PartialEq)]
pub enum Color {
    Red,
    Green,
    Blue,
    White,
    Black,
    Yellow
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    pub symbol: char,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
}

impl Cell {
    pub fn new(symbol: char) -> Self {
        Self {
            symbol,
            fg: None,
            bg: None
        }
    }

    pub fn fg(mut self, color: Option<Color>) -> Self {
        self.fg = color;
        self
    }

    pub fn bg(mut self, color: Option<Color>) -> Self {
        self.bg = color;
        self
    }
}

pub struct Buffer {
    pub width: u16,
    pub height: u16,
    pub cells: Vec<Cell>,
}

impl Buffer {
    pub fn new(width: u16, height: u16) -> Self {
        let size = (width as usize) * (height as usize);
        Self {
            width,
            height,
            cells: vec![Cell::new(' '); size]
        }
    }

    pub fn set_cell(&mut self, x: u16, y: u16, cell: Cell) {
        if x < self.width && y < self.height {
            let index = (y as usize) * (self.width as usize) + (x as usize);
            self.cells[index] = cell;
        }
    }

    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.symbol = ' ';
            cell.fg = None;
            cell.bg = None;
        }
    }
}
