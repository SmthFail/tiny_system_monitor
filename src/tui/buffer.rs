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
    pub ch: char,
    pub fg: Option<Color>,
    pub bg: Option<Color>,
}

impl Cell {
    pub fn new(ch: char) -> Self {
        Self {ch, ..Default::default()}
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

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: None,
            bg: None
        }
    }
}

pub struct Patch {
    pub cell: Cell,
    pub x: usize,
    pub y: usize
}

pub struct Buffer {
    pub cells: Vec<Cell>,
    pub width: u16,
    pub height: u16,
}

impl Buffer {
    pub fn new(width: u16, height: u16) -> Self {
        let cells = vec![Cell::default(); (width * height) as usize];
        Self {width, height, cells}
    }

    pub fn set_cell(&mut self, x: u16, y: u16, new_cell: Cell) {
        let index = (y * self.width + x) as usize;
        if let Some(cell) = self.cells.get_mut(index) {
            *cell = new_cell
        }
    }

    pub fn get_diff(&self, other: &Self) -> Vec<Patch> {
        // TODO handle error when size of buffers not equal
        // assert (self.width == other.width && self.height == other.height)
        let w = self.width as usize;
        self.cells
            .iter()
            .zip(other.cells.iter())
            .enumerate()
            .filter(|(_, (a, b))| *a != *b)
            .map(|(i, (_, cell))| {
                let x = i % w;
                let y = i / w;
                Patch {cell: cell.clone(), x, y}
            })
            .collect()
    }

    pub fn resize(&mut self, w: u16, h: u16) {
        self.cells.resize((w * h) as usize, Cell::default());
        self.clear();
        self.width = w;
        self.height = h;
    }

    pub fn clear(&mut self) {
        self.cells.fill(Cell::default());
    }
}
