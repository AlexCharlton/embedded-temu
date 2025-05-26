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
    pub(crate) dirty: bool,
}

impl Cell {
    /// Get the Cell's character.
    pub fn char(&self) -> char {
        self.c
    }

    /// Get the Cell's foreground color.
    pub fn fg(&self) -> Color {
        self.fg
    }

    /// Get the Cell's background color.
    pub fn bg(&self) -> Color {
        self.bg
    }

    /// Get the Cell's flags.
    pub fn flags(&self) -> Flags {
        self.flags
    }

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
            dirty: true,
        }
    }
}
//-----------------------------------------------------------
// Default cell drawing function
//-----------------------------------------------------------
use embedded_graphics::mono_font::{
    iso_8859_1::{FONT_9X18 as FONT, FONT_9X18_BOLD as FONT_BOLD},
    MonoTextStyleBuilder,
};
use embedded_graphics::prelude::*;
use embedded_graphics::{
    pixelcolor::Rgb888,
    text::{Baseline, Text, TextStyle},
};

/// Draw a cell to a display using the default font, with a fixed color map
pub fn draw_cell_default<D, C: From<Rgb888> + PixelColor>(
    cell: &Cell,
    row: usize,
    col: usize,
    display: &mut D,
) -> Result<(), <D as DrawTarget>::Error>
where
    D: DrawTarget<Color = C>,
{
    let mut utf8_buf = [0u8; 8];
    let s = cell.c.encode_utf8(&mut utf8_buf);
    let (fg, bg) = if cell.flags.contains(Flags::INVERSE) {
        (cell.bg, cell.fg)
    } else {
        (cell.fg, cell.bg)
    };
    let mut style = MonoTextStyleBuilder::new()
        .text_color(C::from(color_to_rgb(fg)))
        .background_color(C::from(color_to_rgb(bg)));
    if cell.flags.contains(Flags::BOLD) {
        style = style.font(&FONT_BOLD);
    } else {
        style = style.font(&FONT);
    }
    if cell.flags.contains(Flags::STRIKEOUT) {
        style = style.strikethrough();
    }
    if cell.flags.contains(Flags::UNDERLINE) {
        style = style.underline();
    }
    let text = Text::with_text_style(
        s,
        Point::new(
            col as i32 * FONT.character_size.width as i32,
            row as i32 * FONT.character_size.height as i32,
        ),
        style.build(),
        TextStyle::with_baseline(Baseline::Top),
    );
    text.draw(display)?;
    Ok(())
}

fn color_to_rgb(color: Color) -> Rgb888 {
    match color {
        Color::RGB(rgb) => rgb,
        Color::Named(name) => COLOR_MAP[name as usize],
        Color::Indexed(idx) => COLOR_MAP[idx as usize],
    }
}

lazy_static::lazy_static! {
    /// Array of indexed colors.
    ///
    /// | Indices  | Description       |
    /// | -------- | ----------------- |
    /// | 0..16    | Named ANSI colors |
    /// | 16..232  | Color cube        |
    /// | 233..256 | Grayscale ramp    |
    ///
    /// Reference: https://en.wikipedia.org/wiki/ANSI_escape_code#Colors
    static ref COLOR_MAP: [Rgb888; 256] = {
        let mut colors = [Rgb888::default(); 256];
        colors[NamedColor::Black as usize] = Rgb888::new(0, 0, 0);
        colors[NamedColor::Red as usize] = Rgb888::new(194, 54, 33);
        colors[NamedColor::Green as usize] = Rgb888::new(37, 188, 36);
        colors[NamedColor::Yellow as usize] = Rgb888::new(173, 173, 39);
        colors[NamedColor::Blue as usize] = Rgb888::new(73, 46, 225);
        colors[NamedColor::Magenta as usize] = Rgb888::new(211, 56, 211);
        colors[NamedColor::Cyan as usize] = Rgb888::new(51, 187, 200);
        colors[NamedColor::White as usize] = Rgb888::new(203, 204, 205);
        colors[NamedColor::BrightBlack as usize] = Rgb888::new(129, 131, 131);
        colors[NamedColor::BrightRed as usize] = Rgb888::new(252, 57, 31);
        colors[NamedColor::BrightGreen as usize] = Rgb888::new(49, 231, 34);
        colors[NamedColor::BrightYellow as usize] = Rgb888::new(234, 236, 35);
        colors[NamedColor::BrightBlue as usize] = Rgb888::new(88, 51, 255);
        colors[NamedColor::BrightMagenta as usize] = Rgb888::new(249, 53, 248);
        colors[NamedColor::BrightCyan as usize] = Rgb888::new(20, 240, 240);
        colors[NamedColor::BrightWhite as usize] = Rgb888::new(233, 235, 235);

        for r in 0..6 {
            for g in 0..6 {
                for b in 0..6 {
                    let index = 16 + 36 * r + 6 * g + b;
                    let f = |c: usize| if c == 0 { 0 } else { (c * 40 + 55) as u8 };
                    colors[index] = Rgb888::new(f(r), f(g), f(b));
                }
            }
        }

        for i in 0..24 {
            let index = 16 + 216 + i;
            let c = (i * 10 + 8) as u8;
            colors[index] = Rgb888::new(c, c, c);
        }

        colors
    };
}
