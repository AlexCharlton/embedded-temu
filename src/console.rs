use crate::Style;
use crate::ansi::{Attr, ClearMode, Handler, LineClearMode, Mode, Performer};
use crate::cell::{Cell, Flags};
use crate::cell_buffer::CellBuffer;
use crate::style::DrawCell;

use alloc::collections::VecDeque;
use core::cmp::min;
use core::fmt;

use embedded_graphics::prelude::*;

use vte::Parser;

/// The primary interface to the terminal emulator.
///
/// Write input strings with control sequences, draw to a [`DrawTarget`].
///
/// Values that are written are encoded as a 2D array of cells, which are then used for drawing with the provided [`Style`].
pub struct Console<'a, C, F> {
    // ANSI escape sequence parser
    parser: Parser,
    // Inner state
    inner: ConsoleInner,
    cell_style: Style<'a, C, F>,
}

#[derive(Debug, Default, Clone, Copy)]
struct Cursor {
    row: usize,
    col: usize,
}

struct ConsoleInner {
    /// cursor
    cursor: Cursor,
    /// Saved cursor
    saved_cursor: Cursor,
    /// current attribute template
    temp: Cell,
    /// character buffer
    buf: CellBuffer,
    /// auto wrap
    auto_wrap: bool,
    /// Reported data for CSI Device Status Report
    report: VecDeque<u8>,
}

impl<'a, C, F> Console<'a, C, F>
where
    Style<'a, C, F>: DrawCell<C>,
{
    /// Create a new console with a given width and height in characters, and a [`Style`]
    pub fn new(width: usize, height: usize, cell_style: Style<'a, C, F>) -> Self {
        Console {
            parser: Parser::new(),
            cell_style,
            inner: ConsoleInner {
                cursor: Cursor::default(),
                saved_cursor: Cursor::default(),
                temp: Cell::default(),
                buf: CellBuffer::new(width, height),
                auto_wrap: true,
                report: VecDeque::new(),
            },
        }
    }

    /// Write a single `byte` to console
    pub fn write_byte(&mut self, byte: u8) {
        self.parser
            .advance(&mut Performer::new(&mut self.inner), byte);
    }

    /// Read result for some commands
    pub fn pop_report(&mut self) -> Option<u8> {
        self.inner.report.pop_front()
    }

    /// Number of rows
    pub fn rows(&self) -> usize {
        self.inner.buf.height()
    }

    /// Number of columns
    pub fn columns(&self) -> usize {
        self.inner.buf.width()
    }

    /// Get the current cursor position
    pub fn get_cursor_position(&self) -> (usize, usize) {
        (self.inner.cursor.row, self.inner.cursor.col)
    }

    #[cfg(feature = "ratatui-backend")]
    pub(crate) fn set_cursor_position(&mut self, row: usize, col: usize) {
        self.inner.goto(row, col);
        self.inner.temp = self.inner.buf.read(row, col);
    }

    #[cfg(feature = "ratatui-backend")]
    pub(crate) fn set_cell(&mut self, row: usize, col: usize, cell: Cell) {
        self.inner.buf.write(row, col, cell);
    }

    /// Draw the console to an embedded-graphics [`DrawTarget`]
    pub fn draw<D, P: PixelColor + From<C>>(
        &mut self,
        display: &mut D,
    ) -> Result<(), <D as DrawTarget>::Error>
    where
        D: DrawTarget<Color = P>,
    {
        for (row, row_cells) in self.inner.buf.buf.iter_mut().enumerate() {
            for (col, cell) in row_cells.iter_mut().enumerate() {
                if cell.dirty {
                    self.cell_style.draw_cell(cell, row, col, display)?;
                    cell.dirty = false;
                }
            }
        }

        Ok(())
    }

    /// Clear the screen
    pub fn clear_screen(&mut self, mode: ClearMode) {
        self.inner.clear_screen(mode);
    }

    /// Clear the line
    pub fn clear_line(&mut self, mode: LineClearMode) {
        self.inner.clear_line(mode);
    }
}

impl<'a, C, F> fmt::Write for Console<'a, C, F>
where
    Style<'a, C, F>: DrawCell<C>,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

impl Handler for ConsoleInner {
    fn input(&mut self, c: char) {
        trace!("  [input]: {:?} @ {:?}", c, self.cursor);
        if self.cursor.col >= self.buf.width() {
            if !self.auto_wrap {
                // skip this one
                return;
            }
            self.cursor.col = 0;
            self.linefeed();
        }
        let mut temp = self.temp;
        temp.c = c;
        self.buf.write(self.cursor.row, self.cursor.col, temp);
        self.cursor.col += 1;
    }

    fn goto(&mut self, row: usize, col: usize) {
        trace!("Going to: line={}, col={}", row, col);
        self.cursor.row = min(row, self.buf.height());
        self.cursor.col = min(col, self.buf.width());
    }

    fn goto_line(&mut self, row: usize) {
        trace!("Going to line: {}", row);
        self.goto(row, self.cursor.col)
    }

    fn goto_col(&mut self, col: usize) {
        trace!("Going to column: {}", col);
        self.goto(self.cursor.row, col)
    }

    fn move_up(&mut self, rows: usize) {
        trace!("Moving up: {}", rows);
        self.goto(self.cursor.row.saturating_sub(rows), self.cursor.col)
    }

    fn move_down(&mut self, rows: usize) {
        trace!("Moving down: {}", rows);
        self.goto(
            min(self.cursor.row + rows, self.buf.height() - 1) as _,
            self.cursor.col,
        )
    }

    fn move_forward(&mut self, cols: usize) {
        trace!("Moving forward: {}", cols);
        self.cursor.col = min(self.cursor.col + cols, self.buf.width() - 1);
    }

    fn move_backward(&mut self, cols: usize) {
        trace!("Moving backward: {}", cols);
        self.cursor.col = self.cursor.col.saturating_sub(cols);
    }

    fn move_down_and_cr(&mut self, rows: usize) {
        trace!("Moving down and cr: {}", rows);
        self.goto(min(self.cursor.row + rows, self.buf.height() - 1) as _, 0)
    }

    fn move_up_and_cr(&mut self, rows: usize) {
        trace!("Moving up and cr: {}", rows);
        self.goto(self.cursor.row.saturating_sub(rows), 0)
    }

    fn put_tab(&mut self, count: u16) {
        let mut count = count;
        let bg = self.temp.just_bg();
        while self.cursor.col < self.buf.width() && count > 0 {
            count -= 1;
            loop {
                self.buf.write(self.cursor.row, self.cursor.col, bg);
                self.cursor.col += 1;
                if self.cursor.col == self.buf.width() || self.cursor.col % 8 == 0 {
                    break;
                }
            }
        }
    }

    fn backspace(&mut self) {
        trace!("Backspace");
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        }
    }

    fn carriage_return(&mut self) {
        trace!("Carriage return");
        self.cursor.col = 0;
    }

    fn linefeed(&mut self) {
        trace!("Linefeed");
        self.cursor.col = 0;
        if self.cursor.row < self.buf.height() - 1 {
            self.cursor.row += 1;
        } else {
            self.buf.new_line(self.temp);
        }
    }

    fn scroll_up(&mut self, rows: usize) {
        debug!("[Unhandled CSI] scroll_up {:?}", rows);
    }

    fn scroll_down(&mut self, rows: usize) {
        debug!("[Unhandled CSI] scroll_down {:?}", rows);
    }

    fn erase_chars(&mut self, count: usize) {
        trace!("Erasing chars: count={}, col={}", count, self.cursor.col);

        let start = self.cursor.col;
        let end = min(start + count, self.buf.width());

        // Cleared cells have current background color set.
        let bg = self.temp.just_bg();
        for i in start..end {
            self.buf.write(self.cursor.row, i, bg);
        }
    }
    fn delete_chars(&mut self, count: usize) {
        let columns = self.buf.width();
        let count = min(count, columns - self.cursor.col - 1);
        let row = self.cursor.row;

        let start = self.cursor.col;
        let end = start + count;

        let bg = self.temp.just_bg();
        for i in end..columns {
            self.buf.write(row, i - count, self.buf.read(row, i));
            self.buf.write(row, i, bg);
        }
    }

    /// Save current cursor position.
    fn save_cursor_position(&mut self) {
        trace!("Saving cursor position");
        self.saved_cursor = self.cursor;
    }

    /// Restore cursor position.
    fn restore_cursor_position(&mut self) {
        trace!("Restoring cursor position");
        self.cursor = self.saved_cursor;
    }

    fn clear_line(&mut self, mode: LineClearMode) {
        trace!("Clearing line: {:?}", mode);
        let bg = self.temp.just_bg();
        match mode {
            LineClearMode::Right => {
                for i in self.cursor.col..self.buf.width() {
                    self.buf.write(self.cursor.row, i, bg);
                }
            }
            LineClearMode::Left => {
                for i in 0..=self.cursor.col {
                    self.buf.write(self.cursor.row, i, bg);
                }
            }
            LineClearMode::All => {
                for i in 0..self.buf.width() {
                    self.buf.write(self.cursor.row, i, bg);
                }
            }
        }
    }

    fn clear_screen(&mut self, mode: ClearMode) {
        trace!("Clearing screen: {:?}", mode);
        let bg = self.temp.just_bg();
        let row = self.cursor.row;
        let col = self.cursor.col;
        match mode {
            ClearMode::Above => {
                for i in 0..row {
                    for j in 0..self.buf.width() {
                        self.buf.write(i, j, bg);
                    }
                }
                for j in 0..col {
                    self.buf.write(row, j, bg);
                }
            }
            ClearMode::Below => {
                for j in col..self.buf.width() {
                    self.buf.write(row, j, bg);
                }
                for i in row + 1..self.buf.height() {
                    for j in 0..self.buf.width() {
                        self.buf.write(i, j, bg);
                    }
                }
            }
            ClearMode::All => {
                self.buf.clear(bg);
                self.cursor = Cursor::default();
            }
            _ => {}
        }
    }

    fn terminal_attribute(&mut self, attr: Attr) {
        trace!("Setting attribute: {:?}", attr);
        match attr {
            Attr::Foreground(color) => self.temp.fg = color,
            Attr::Background(color) => self.temp.bg = color,
            Attr::Reset => self.temp = Cell::default(),
            Attr::Reverse => self.temp.flags |= Flags::INVERSE,
            Attr::CancelReverse => self.temp.flags.remove(Flags::INVERSE),
            Attr::Bold => self.temp.flags.insert(Flags::BOLD),
            Attr::CancelBold => self.temp.flags.remove(Flags::BOLD),
            Attr::Dim => self.temp.flags.insert(Flags::DIM),
            Attr::CancelBoldDim => self.temp.flags.remove(Flags::BOLD | Flags::DIM),
            Attr::Italic => self.temp.flags.insert(Flags::ITALIC),
            Attr::CancelItalic => self.temp.flags.remove(Flags::ITALIC),
            Attr::Underline => self.temp.flags.insert(Flags::UNDERLINE),
            Attr::CancelUnderline => self.temp.flags.remove(Flags::UNDERLINE),
            Attr::Hidden => self.temp.flags.insert(Flags::HIDDEN),
            Attr::CancelHidden => self.temp.flags.remove(Flags::HIDDEN),
            Attr::Strike => self.temp.flags.insert(Flags::STRIKEOUT),
            Attr::CancelStrike => self.temp.flags.remove(Flags::STRIKEOUT),
            _ => {
                debug!("Term got unhandled attr: {:?}", attr);
            }
        }
    }

    fn set_mode(&mut self, mode: Mode) {
        if mode == Mode::LineWrap {
            self.auto_wrap = true;
        } else {
            debug!("[Unhandled CSI] Setting mode: {:?}", mode);
        }
    }

    fn unset_mode(&mut self, mode: Mode) {
        if mode == Mode::LineWrap {
            self.auto_wrap = false;
        } else {
            debug!("[Unhandled CSI] Setting mode: {:?}", mode);
        }
    }

    fn set_scrolling_region(&mut self, top: usize, bottom: Option<usize>) {
        let bottom = bottom.unwrap_or_else(|| self.buf.height());
        debug!(
            "[Unhandled CSI] Setting scrolling region: ({};{})",
            top, bottom
        );
    }

    fn device_status(&mut self, arg: usize) {
        trace!("Reporting device status: {}", arg);
        match arg {
            5 => {
                for &c in b"\x1b[0n" {
                    self.report.push_back(c);
                }
            }
            6 => {
                let s = alloc::format!("\x1b[{};{}R", self.cursor.row + 1, self.cursor.col + 1);
                for c in s.bytes() {
                    self.report.push_back(c);
                }
            }
            _ => debug!("unknown device status query: {}", arg),
        }
    }
}
