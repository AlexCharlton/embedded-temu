use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::{DrawTarget, PixelColor};
use ratatui::backend::{ClearType, WindowSize};
use ratatui::buffer::Cell as RatatuiCell;
use ratatui::prelude::{Position, Size};
use ratatui::style::{Color as RatatuiColor, Modifier as RatatuiModifier};

use crate::ansi::{ClearMode, LineClearMode};
use crate::cell::{Cell, Flags};
use crate::color::{Color, NamedColor};
use crate::style::{ColorInterpolate, DrawCell, Style};

/// A set of glyphs to be used by [`Mono8BitFont`] that contains the border characters Ratatui uses.
///
/// Includes ASCII characters and the [box drawing and block element characters](https://en.wikipedia.org/wiki/Box-drawing_characters).
pub const RATATUI_GLYPHS: &'static str = "\0\u{20}\u{7e}\0\u{2500}\u{259f}";

/// A [`ratatui::backend::Backend`] implementation for the Embedded Temu
pub struct EmbeddedTemuBackend<'a, C, E, P, FD: FlushableDisplay<E, P>, F> {
    console: crate::Console<'a, C, F>,
    display: FD,
    _marker: core::marker::PhantomData<E>,
    _marker2: core::marker::PhantomData<P>,
}

impl<'a, C, E, P, FD: FlushableDisplay<E, P>, F> EmbeddedTemuBackend<'a, C, E, P, FD, F> {
    /// Create a new [`EmbeddedTemuBackend`]
    pub fn new(console: crate::Console<'a, C, F>, display: FD) -> Self {
        Self {
            console,
            display,
            _marker: core::marker::PhantomData,
            _marker2: core::marker::PhantomData,
        }
    }
}

/// A trait for displays that can be flushed
pub trait FlushableDisplay<E, C>: DrawTarget<Error = E, Color = C> {
    /// Flush the display
    fn flush(&mut self) -> Result<(), E>;
}

/// Errors that can occur when using the [`EmbeddedTemuBackend`]
#[derive(Debug)]
pub enum BackendError<E: core::fmt::Debug> {
    /// The cursor position is out of bounds
    CursorPositionOutOfBounds,
    /// The flush operation failed
    FlushError(E),
}

impl<E: core::fmt::Display + core::fmt::Debug> core::fmt::Display for BackendError<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<E: core::fmt::Display + core::fmt::Debug> core::error::Error for BackendError<E> {}

impl<
    'a,
    C,
    E: core::fmt::Display + core::fmt::Debug,
    P: PixelColor + From<C> + ColorInterpolate,
    FD: FlushableDisplay<E, P>,
    F,
> ratatui::backend::Backend for EmbeddedTemuBackend<'a, C, E, P, FD, F>
where
    Style<'a, C, F>: DrawCell<C>,
{
    type Error = BackendError<E>;

    fn draw<'b, I>(&mut self, content: I) -> Result<(), Self::Error>
    where
        I: Iterator<Item = (u16, u16, &'b RatatuiCell)>,
    {
        trace!("Drawing to console");
        let rows = self.console.rows();
        let cols = self.console.columns();
        for (x, y, cell) in content {
            if x > cols as u16 || y > rows as u16 {
                return Err(BackendError::CursorPositionOutOfBounds);
            }
            debug!("Setting cell: {:?}", cell);
            self.console
                .set_cell(y as usize, x as usize, ratatui_cell_to_cell(cell));
            self.console.set_cursor_position(x as usize, y as usize);
        }
        Ok(())
    }

    // Cursor is never shown
    fn hide_cursor(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    // Cursor is never shown
    fn show_cursor(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn get_cursor_position(&mut self) -> Result<Position, Self::Error> {
        let (row, col) = self.console.get_cursor_position();
        Ok(Position::new(col as u16, row as u16))
    }

    fn set_cursor_position<POS: Into<Position>>(
        &mut self,
        position: POS,
    ) -> Result<(), Self::Error> {
        let position = position.into();
        if position.x > self.console.columns() as u16 || position.y > self.console.rows() as u16 {
            return Err(BackendError::CursorPositionOutOfBounds);
        }
        self.console
            .set_cursor_position(position.x as usize, position.y as usize);
        Ok(())
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        self.console.clear_screen(ClearMode::All);
        Ok(())
    }

    fn clear_region(&mut self, clear_type: ClearType) -> Result<(), Self::Error> {
        match clear_type {
            ClearType::All => self.console.clear_screen(ClearMode::All),
            ClearType::AfterCursor => self.console.clear_screen(ClearMode::Below),
            ClearType::BeforeCursor => self.console.clear_screen(ClearMode::Above),
            ClearType::CurrentLine => self.console.clear_line(LineClearMode::All),
            ClearType::UntilNewLine => self.console.clear_line(LineClearMode::Right),
        }
        Ok(())
    }

    fn size(&self) -> Result<Size, Self::Error> {
        Ok(Size::new(
            self.console.columns() as u16,
            self.console.rows() as u16,
        ))
    }

    fn window_size(&mut self) -> Result<WindowSize, Self::Error> {
        let size = self.display.bounding_box().size;
        Ok(WindowSize {
            pixels: Size::new(size.width as u16, size.height as u16),
            columns_rows: Size::new(self.console.columns() as u16, self.console.rows() as u16),
        })
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        trace!("Flushing display");
        self.console
            .draw(&mut self.display)
            .map_err(|e| BackendError::FlushError(e))?;
        self.display
            .flush()
            .map_err(|e| BackendError::FlushError(e))?;
        Ok(())
    }
}

//--------------------------------
// Ratatui conversions

fn ratatui_cell_to_cell(cell: &RatatuiCell) -> Cell {
    Cell {
        // Maybe TODO; handle multi-character symbols
        c: cell.symbol().chars().next().unwrap(),
        fg: ratatui_color_to_color(&cell.fg, false),
        bg: ratatui_color_to_color(&cell.bg, true),
        flags: ratatui_modifier_to_flags(&cell.modifier),
        dirty: true,
    }
}

fn ratatui_color_to_color(color: &RatatuiColor, bg: bool) -> Color {
    match color {
        RatatuiColor::Reset => {
            if bg {
                Color::Named(NamedColor::Black)
            } else {
                Color::Named(NamedColor::BrightWhite)
            }
        }
        RatatuiColor::Black => Color::Named(NamedColor::Black),
        RatatuiColor::Red => Color::Named(NamedColor::Red),
        RatatuiColor::Green => Color::Named(NamedColor::Green),
        RatatuiColor::Yellow => Color::Named(NamedColor::Yellow),
        RatatuiColor::Blue => Color::Named(NamedColor::Blue),
        RatatuiColor::Magenta => Color::Named(NamedColor::Magenta),
        RatatuiColor::Cyan => Color::Named(NamedColor::Cyan),
        RatatuiColor::White => Color::Named(NamedColor::BrightWhite),
        RatatuiColor::Gray => Color::Named(NamedColor::White),
        RatatuiColor::DarkGray => Color::Named(NamedColor::BrightBlack),
        RatatuiColor::LightRed => Color::Named(NamedColor::BrightRed),
        RatatuiColor::LightGreen => Color::Named(NamedColor::BrightGreen),
        RatatuiColor::LightYellow => Color::Named(NamedColor::BrightYellow),
        RatatuiColor::LightBlue => Color::Named(NamedColor::BrightBlue),
        RatatuiColor::LightMagenta => Color::Named(NamedColor::BrightMagenta),
        RatatuiColor::LightCyan => Color::Named(NamedColor::BrightCyan),
        RatatuiColor::Rgb(r, g, b) => Color::RGB(Rgb888::new(*r, *g, *b)),
        RatatuiColor::Indexed(i) => Color::Indexed(*i),
    }
}

fn ratatui_modifier_to_flags(modifier: &RatatuiModifier) -> Flags {
    let mut flags = Flags::empty();
    if modifier.contains(RatatuiModifier::BOLD) {
        flags.insert(Flags::BOLD);
    }
    if modifier.contains(RatatuiModifier::DIM) {
        flags.insert(Flags::DIM);
    }
    if modifier.contains(RatatuiModifier::ITALIC) {
        flags.insert(Flags::ITALIC);
    }
    if modifier.contains(RatatuiModifier::UNDERLINED) {
        flags.insert(Flags::UNDERLINE);
    }
    if modifier.contains(RatatuiModifier::REVERSED) {
        flags.insert(Flags::INVERSE);
    }
    if modifier.contains(RatatuiModifier::HIDDEN) {
        flags.insert(Flags::HIDDEN);
    }
    if modifier.contains(RatatuiModifier::CROSSED_OUT) {
        flags.insert(Flags::STRIKEOUT);
    }
    flags
}
