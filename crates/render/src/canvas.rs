use crate::cell::Cell;

/// Represents the canvas where everything is rendered
/// It keeps a vector of all the cells, and the settings of the render
#[derive(Debug, Clone)]
pub struct Canvas {
    cells: Vec<Cell>,
    pub width: usize,
    pub height: usize,
}

impl Canvas {
    /// Creates a canvas from a [`Settings`]
    /// Sets the cells vector to the correct size with a default [`Cell`]
    pub fn new(width: usize, height: usize) -> Canvas {
        Canvas {
            cells: vec![Cell::default(); width * height],
            width,
            height,
        }
    }

    fn idx(&self, x: usize, y: usize) -> Option<usize> {
        (x < self.width && y < self.height && self.cells.len() > y * self.width + x)
            .then_some(y * self.width + x)
    }

    /// Get cell at position (x,y) returns `None` if either cells is not big enough or (x,y) is too big for the width and height.
    pub fn get_cell(&self, x: usize, y: usize) -> Option<&Cell> {
        self.idx(x, y).map(|i| &self.cells[i])
    }

    /// See get_cell
    pub fn get_mut_cell(&mut self, x: usize, y: usize) -> Option<&mut Cell> {
        self.idx(x, y).map(|i| &mut self.cells[i])
    }

    /// Sets a cell at (x,y) returns true if succesful and false if the (x,y) given too big for cells
    pub fn set(&mut self, x: usize, y: usize, cell: Cell) -> bool {
        if let Some(i) = self.idx(x, y) {
            if i >= self.cells.len() {
                self.cells.resize_with(i + 1, Cell::default);
            }
            self.cells[i] = cell;
            true
        } else {
            false
        }
    }
}
