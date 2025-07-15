use crate::color::{Color, NamedColor};

bitflags::bitflags! {
    /// Bit flags for graphical rendition, corresponding to [select ANSI escape parameters](https://en.wikipedia.org/wiki/ANSI_escape_code#Select_Graphic_Rendition_parameters). See [`bitflags`] for usage information.
    pub struct Flags: u16 {
        /// Invert foreground and background colors.
        const INVERSE                   = 0b0000_0000_0000_0001;
        /// Bold text.
        const BOLD                      = 0b0000_0000_0000_0010;
        /// Italic text.
        const ITALIC                    = 0b0000_0000_0000_0100;
        /// Bold and italic text.
        const BOLD_ITALIC               = 0b0000_0000_0000_0110;
        /// Underline text.
        const UNDERLINE                 = 0b0000_0000_0000_1000;
        /// Wrap line.
        const WRAPLINE                  = 0b0000_0000_0001_0000;
        /// Wide character.
        const WIDE_CHAR                 = 0b0000_0000_0010_0000;
        /// Spacer for wide characters.
        const WIDE_CHAR_SPACER          = 0b0000_0000_0100_0000;
        /// Dim text.
        const DIM                       = 0b0000_0000_1000_0000;
        /// Dim and bold text.
        const DIM_BOLD                  = 0b0000_0000_1000_0010;
        /// Hidden text.
        const HIDDEN                    = 0b0000_0001_0000_0000;
        /// Strikeout text.
        const STRIKEOUT                 = 0b0000_0010_0000_0000;
        /// Leading wide character spacer.
        const LEADING_WIDE_CHAR_SPACER  = 0b0000_0100_0000_0000;
        /// Double underline text.
        const DOUBLE_UNDERLINE          = 0b0000_1000_0000_0000;
    }
}

/// A character on the screen
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Cell {
    pub(crate) c: char,
    pub(crate) fg: Color,
    pub(crate) bg: Color,
    pub(crate) flags: Flags,
    // Number of times we need to flush this cell
    pub(crate) to_flush: usize,
}

impl Cell {
    pub(crate) fn just_bg(&self) -> Self {
        Self {
            bg: self.bg,
            ..Default::default()
        }
    }
}

impl Default for Cell {
    #[inline]
    fn default() -> Cell {
        Cell {
            c: ' ',
            bg: Color::Named(NamedColor::Black),
            fg: Color::Named(NamedColor::BrightWhite),
            flags: Flags::empty(),
            to_flush: 1,
        }
    }
}
