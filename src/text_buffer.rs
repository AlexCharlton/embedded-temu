use crate::cell::Cell;
use alloc::vec::Vec;

/// A 2D array of `Cell` to render on screen
pub struct TextBuffer {
    pub buf: Vec<Vec<Cell>>,
    row_offset: usize,
    width: usize,
    height: usize,
}

impl TextBuffer {
    /// Create a new text buffer
    pub fn new(width: usize, height: usize) -> Self {
        TextBuffer {
            buf: vec![vec![Cell::default(); width]; height],
            row_offset: 0,
            width,
            height,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    /// Read the character at `(row, col)`
    pub fn read(&self, row: usize, col: usize) -> Cell {
        if row >= self.height() || col >= self.width() {
            return Cell::default();
        }
        self.buf[row][col]
    }

    /// Write a character `ch` at `(row, col)`
    pub fn write(&mut self, row: usize, col: usize, cell: Cell) {
        if row >= self.height() || col >= self.width() {
            return;
        }
        self.buf[row][col] = cell;
    }

    /// Insert one blank line at the bottom, and scroll up one line.
    pub fn new_line(&mut self, cell: Cell) {
        self.clear_line(self.row_offset, cell);
        self.row_offset = (self.row_offset + 1) % self.height();
    }

    /// Clear line at `row`
    fn clear_line(&mut self, row: usize, cell: Cell) {
        for col in 0..self.width() {
            self.buf[row][col] = cell;
        }
    }

    pub fn clear(&mut self, cell: Cell) {
        self.row_offset = 0;
        for i in 0..self.height() {
            for j in 0..self.width() {
                self.write(i, j, cell);
            }
        }
    }
}
