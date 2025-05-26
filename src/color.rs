pub use embedded_graphics::pixelcolor::Rgb888;

/// Standard colors, by name.
//
// The order here matters since the enum should be castable to a `usize` for
// indexing a color list.
#[allow(missing_docs)]
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum NamedColor {
    Black = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    White = 7,
    BrightBlack = 8,
    BrightRed = 9,
    BrightGreen = 10,
    BrightYellow = 11,
    BrightBlue = 12,
    BrightMagenta = 13,
    BrightCyan = 14,
    BrightWhite = 15,
}

/// A color. Can take the form of a named color, a specific RGB color, or an
/// indexed color. See [ANSI escape code](https://en.wikipedia.org/wiki/ANSI_escape_code#Colors)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    /// A named color.
    Named(NamedColor),
    /// A specific RGB color.
    RGB(Rgb888),
    /// An indexed color.
    Indexed(u8),
}
