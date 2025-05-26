use crate::cell::{Cell, Flags};
use crate::color::{Color, NamedColor};

use embedded_graphics::mono_font::{
    iso_8859_1::{FONT_9X18 as FONT, FONT_9X18_BOLD as FONT_BOLD},
    MonoFont, MonoTextStyleBuilder,
};
use embedded_graphics::prelude::*;
use embedded_graphics::{
    pixelcolor::Rgb888,
    text::{Baseline, Text, TextStyle},
};

/// A style for drawing the [`Console`][crate::Console].
///
/// This is used to configure the font and color mapping.
pub struct Style<C> {
    /// The base font to use for the cell.
    pub font: &'static MonoFont<'static>,
    /// The bold font to use for the cell.
    pub font_bold: &'static MonoFont<'static>,
    /// A function to convert a [`Color`] to a value that can be converted to a given [`DrawTarget`]'s [`PixelColor`] (i.e. implements [`From`])
    pub color_to_pixel: fn(Color) -> C,
}

impl<C> Style<C> {
    fn color_to_pixel(&self, color: Color) -> C {
        (self.color_to_pixel)(color)
    }
}

// Draw a cell to a display using the provided style
pub(crate) fn draw_cell<D, C, P: PixelColor + From<C>>(
    cell: &Cell,
    row: usize,
    col: usize,
    display: &mut D,
    cell_style: &Style<C>,
) -> Result<(), <D as DrawTarget>::Error>
where
    D: DrawTarget<Color = P>,
{
    let mut utf8_buf = [0u8; 8];
    let s = cell.c.encode_utf8(&mut utf8_buf);
    let (fg, bg) = if cell.flags.contains(Flags::INVERSE) {
        (cell.bg, cell.fg)
    } else {
        (cell.fg, cell.bg)
    };
    let mut style = MonoTextStyleBuilder::new()
        .text_color(P::from(cell_style.color_to_pixel(fg)))
        .background_color(P::from(cell_style.color_to_pixel(bg)));
    if cell.flags.contains(Flags::BOLD) {
        style = style.font(cell_style.font_bold);
    } else {
        style = style.font(cell_style.font);
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
            col as i32 * cell_style.font.character_size.width as i32,
            row as i32 * cell_style.font.character_size.height as i32,
        ),
        style.build(),
        TextStyle::with_baseline(Baseline::Top),
    );
    text.draw(display)?;
    Ok(())
}

//-----------------------------------------------------------
// Default cell styling
//-----------------------------------------------------------
impl Default for Style<Rgb888> {
    fn default() -> Self {
        Self {
            font: &FONT,
            font_bold: &FONT_BOLD,
            color_to_pixel: |color| color_to_rgb(color),
        }
    }
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
