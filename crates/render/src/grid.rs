use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid<T> {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<T>,
}

impl<T> Grid<T> {
    /// Creates a new grid filled by calling a closure for each `(x, y)` coordinate.
    pub fn new_with<F>(width: usize, height: usize, mut f: F) -> Self
    where
        F: FnMut(usize, usize) -> T,
    {
        let mut cells = Vec::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                cells.push(f(x, y));
            }
        }
        Self {
            width,
            height,
            cells,
        }
    }

    pub fn vec_to_grid(cells: Vec<T>, width: usize, height: usize) -> Option<Self> {
        if width * height != cells.len() {
            None
        } else {
            Some(Grid {
                cells,
                width,
                height,
            })
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    /// Converts `(x, y)` coordinates into a flat vector index, returning `None` if out of bounds.
    pub fn index_of(&self, x: usize, y: usize) -> Option<usize> {
        if x < self.width && y < self.height {
            Some(y * self.width + x)
        } else {
            None
        }
    }

    /// Returns a reference to the cell at `(x, y)`, or `None` if out of bounds.
    pub fn get(&self, x: usize, y: usize) -> Option<&T> {
        self.index_of(x, y).map(|idx| &self.cells[idx])
    }

    /// Returns a mutable reference to the cell at `(x, y)`, or `None` if out of bounds.
    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.index_of(x, y).map(|idx| &mut self.cells[idx])
    }

    /// Iterates over all cells with their `(x, y)` coordinates in row-major order.
    pub fn iter(&self) -> impl Iterator<Item = (usize, usize, &T)> {
        self.cells
            .iter()
            .enumerate()
            .map(|(idx, cell)| (idx % self.width, idx / self.width, cell))
    }

    /// Map on a grid guarantees that the Grid stays the same width and height
    pub fn map<U>(&self, f: impl Fn(&T) -> U) -> Grid<U> {
        let cells: Vec<U> = self.iter().map(|(_, _, t)| f(t)).collect();
        Grid {
            cells,
            width: self.width(),
            height: self.height(),
        }
    }

    /// Map with a closure with (x,y,T) as input, guarantees that the Grid stays the same width and height
    pub fn map_indexed<U>(&self, f: impl Fn((usize, usize, &T)) -> U) -> Grid<U> {
        let cells: Vec<U> = self.iter().map(f).collect();
        Grid {
            cells,
            width: self.width(),
            height: self.height(),
        }
    }

    /// Transforms an iter back to a Grid, gives an Option back as we can't guarantee sizes fit correctly
    pub fn from_iter_with_dims<I>(iter: I, width: usize, height: usize) -> Option<Self>
    where
        I: IntoIterator<Item = T>,
    {
        let cells: Vec<T> = iter.into_iter().collect();
        if cells.len() == width * height {
            Some(Self {
                cells,
                width,
                height,
            })
        } else {
            None
        }
    }
}

impl<T: Clone> Grid<T> {
    /// Creates a new grid filled with clones of `value`.
    pub fn new(width: usize, height: usize, value: T) -> Self {
        Self {
            width,
            height,
            cells: vec![value; width * height],
        }
    }
}

impl<T: Default + Clone> Grid<T> {
    /// Creates a new grid filled with `T::default()`.
    pub fn new_default(width: usize, height: usize) -> Self {
        Self::new(width, height, T::default())
    }
}

/// Enables ergonomic `grid[(x, y)]` read access (panics if out of bounds).
impl<T> Index<(usize, usize)> for Grid<T> {
    type Output = T;

    fn index(&self, (x, y): (usize, usize)) -> &Self::Output {
        let idx = self.index_of(x, y).expect("grid index out of bounds");
        &self.cells[idx]
    }
}

/// Enables ergonomic `grid[(x, y)] = val` write access (panics if out of bounds).
impl<T> IndexMut<(usize, usize)> for Grid<T> {
    fn index_mut(&mut self, (x, y): (usize, usize)) -> &mut Self::Output {
        let idx = self.index_of(x, y).expect("grid index out of bounds");
        &mut self.cells[idx]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_of_and_bounds() {
        let grid = Grid::new(3, 2, 0);
        assert_eq!(grid.index_of(2, 1), Some(5));
        assert_eq!(grid.index_of(3, 0), None);
        assert_eq!(grid.index_of(0, 2), None);
    }

    #[test]
    fn test_new_with_is_row_major() {
        let grid = Grid::new_with(3, 2, |x, y| (x, y));
        assert_eq!(grid.cells[4], (1, 1));
        assert!(grid.iter().all(|(x, y, &cell)| cell == (x, y)));
    }

    #[test]
    fn test_index_mut_writes_correct_cell() {
        let mut grid = Grid::new(3, 2, 0);
        grid[(1, 1)] = 7;
        assert_eq!(grid.cells[4], 7);
        assert_eq!(grid.get(1, 1), Some(&7));
    }

    #[test]
    #[should_panic]
    fn test_index_out_of_bounds_panics() {
        let grid = Grid::new(3, 2, 0);
        let _ = grid[(3, 0)];
    }

    #[test]
    fn test_map_keeps_dimensions() {
        let grid = Grid::new_with(3, 2, |x, y| x + y);
        let mapped = grid.map_indexed(|(x, y, v)| x * 10 + y + v);
        assert_eq!((mapped.width, mapped.height), (3, 2));
        assert_eq!(mapped[(2, 1)], 24);
    }

    #[test]
    fn test_from_iter_with_dims_checks_length() {
        assert!(Grid::from_iter_with_dims(0..6, 3, 2).is_some());
        assert!(Grid::from_iter_with_dims(0..5, 3, 2).is_none());
        assert!(Grid::vec_to_grid(vec![0; 5], 3, 2).is_none());
    }
}
