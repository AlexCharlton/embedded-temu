use crate::cell::{Cell, Flags};
use crate::color::{Color, NamedColor};

use embedded_graphics::mono_font::{
    MonoFont, MonoTextStyleBuilder,
    iso_8859_1::{FONT_9X18 as FONT, FONT_9X18_BOLD as FONT_BOLD},
};
use embedded_graphics::prelude::*;
use embedded_graphics::{
    pixelcolor::{Rgb666, Rgb888},
    text::{Baseline, Text, TextStyle},
};

//-----------------------------------------------------------
// MARK: Color interpolation
//-----------------------------------------------------------

/// A trait for [`PixelColor`] types to allow them to interpolate between two colors. This is only used for fontdue text rendering, but we're going to require it for all [`PixelColor`] types that are being drawn to a display since we require a DrawCell bound do be present.
pub trait ColorInterpolate: PixelColor + Sized + Copy {
    /// Interpolate between two colors by the amount specified in the value. 0 is fully background color, 255 is fully foreground color.
    fn interpolate(fg: Self, bg: Self, value: u8) -> Self;
}

impl ColorInterpolate for Rgb666 {
    fn interpolate(fg: Self, bg: Self, value: u8) -> Self {
        let r = interpolate_8bit_values(bg.r(), fg.r(), value);
        let g = interpolate_8bit_values(bg.g(), fg.g(), value);
        let b = interpolate_8bit_values(bg.b(), fg.b(), value);
        Self::new(r, g, b)
    }
}

impl ColorInterpolate for Rgb888 {
    fn interpolate(fg: Self, bg: Self, value: u8) -> Self {
        let r = interpolate_8bit_values(bg.r(), fg.r(), value);
        let g = interpolate_8bit_values(bg.g(), fg.g(), value);
        let b = interpolate_8bit_values(bg.b(), fg.b(), value);
        Self::new(r, g, b)
    }
}

/// Interpolate between two 8-bit values by the amount specified in the value. 0 is fully background color, 255 is fully foreground color.
pub fn interpolate_8bit_values(a: u8, b: u8, value: u8) -> u8 {
    let a = a as u16;
    let b = b as u16;
    let value = value as u16;

    let result = if a > b {
        a - ((a - b) * value + 127) / 255
    } else {
        a + ((b - a) * value + 127) / 255
    };

    result as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolate_color_values() {
        // Edge cases
        assert_eq!(interpolate_8bit_values(0, 0, 0), 0, "0% between 0 and 0");
        assert_eq!(
            interpolate_8bit_values(255, 255, 255),
            255,
            "100% between 255 and 255"
        );
        assert_eq!(
            interpolate_8bit_values(0, 255, 0),
            0,
            "0% between bg:0 and fg:255"
        );
        assert_eq!(
            interpolate_8bit_values(0, 255, 255),
            255,
            "100% between bg:0 and fg:255"
        );

        // 50% interpolation
        assert_eq!(
            interpolate_8bit_values(0, 255, 128),
            128,
            "50% between bg:0 and fg:255"
        );
        assert_eq!(
            interpolate_8bit_values(255, 0, 128),
            127,
            "50% between bg:255 and fg:0"
        );

        // 25% and 75% interpolation
        assert_eq!(
            interpolate_8bit_values(0, 255, 64),
            64,
            "25% between bg:0 and fg:255"
        );
        assert_eq!(
            interpolate_8bit_values(0, 255, 192),
            192,
            "75% between bg:0 and fg:255"
        );

        // Arbitrary values
        assert_eq!(
            interpolate_8bit_values(100, 200, 128),
            150,
            "50% between bg:100 and fg:200"
        );
        assert_eq!(
            interpolate_8bit_values(50, 150, 128),
            100,
            "50% between bg:50 and fg:150"
        );
    }
}

//-----------------------------------------------------------
// MARK: DrawCell trait
//-----------------------------------------------------------

/// A trait for types that can draw cells
pub trait DrawCell<C> {
    fn draw_cell<D, P>(
        &self,
        cell: &Cell,
        row: usize,
        col: usize,
        display: &mut D,
    ) -> Result<(), <D as DrawTarget>::Error>
    where
        D: DrawTarget<Color = P>,
        P: PixelColor + From<C> + ColorInterpolate;
}

//-----------------------------------------------------------
// MARK: Style
//-----------------------------------------------------------

/// A style for drawing the [`Console`][crate::Console].
///
/// This is used to configure the font and color mapping.
pub struct Style<'a, C, F> {
    /// The base font to use for the cell.
    pub font: &'a F,
    /// The bold font to use for the cell.
    pub font_bold: &'a F,
    /// A function to convert a [`Color`] to a value that can be converted to a given [`DrawTarget`]'s [`PixelColor`] (i.e. implements [`From`])
    pub color_to_pixel: fn(Color) -> C,
    /// A function to dim a color
    pub dim_color: fn(C) -> C,
    /// Pixel amount to offset all cells by
    pub offset: (u32, u32),
}

impl<'a, C, F> Style<'a, C, F> {
    /// Create a new [`Style`].
    pub fn new(
        font: &'a F,
        font_bold: &'a F,
        color_to_pixel: fn(Color) -> C,
        dim_color: fn(C) -> C,
    ) -> Self {
        Self {
            font,
            font_bold,
            color_to_pixel,
            dim_color,
            offset: (0, 0),
        }
    }

    /// Call the `color_to_pixel` function.
    pub fn color_to_pixel(&self, color: Color) -> C {
        (self.color_to_pixel)(color)
    }

    /// Call the `dim_color` function.
    pub fn dim_color(&self, color: C) -> C {
        (self.dim_color)(color)
    }
}

//-----------------------------------------------------------
// MARK: MonoFont DrawCell implementation
//-----------------------------------------------------------

impl<C> DrawCell<C> for Style<'static, C, MonoFont<'static>> {
    fn draw_cell<D, P: PixelColor + From<C>>(
        &self,
        cell: &Cell,
        row: usize,
        col: usize,
        display: &mut D,
    ) -> Result<(), <D as DrawTarget>::Error>
    where
        D: DrawTarget<Color = P>,
    {
        info!("Drawing cell: {:?}", cell);
        let mut utf8_buf = [0u8; 8];
        let s = cell.c.encode_utf8(&mut utf8_buf);
        let (fg, bg) = if cell.flags.contains(Flags::INVERSE) {
            (cell.bg, cell.fg)
        } else {
            (cell.fg, cell.bg)
        };
        let mut fg = self.color_to_pixel(fg);
        let mut bg = self.color_to_pixel(bg);
        if cell.flags.contains(Flags::DIM) {
            fg = self.dim_color(fg);
            bg = self.dim_color(bg);
        }
        let mut style = MonoTextStyleBuilder::new()
            .text_color(P::from(fg))
            .background_color(P::from(bg));
        if cell.flags.contains(Flags::BOLD) {
            style = style.font(self.font_bold);
        } else {
            style = style.font(self.font);
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
                col as i32 * self.font.character_size.width as i32 + self.offset.0 as i32,
                row as i32 * self.font.character_size.height as i32 + self.offset.1 as i32,
            ),
            style.build(),
            TextStyle::with_baseline(Baseline::Top),
        );
        text.draw(display)?;
        Ok(())
    }
}

//-----------------------------------------------------------
// MARK:Default cell styling
//-----------------------------------------------------------
impl Default for Style<'static, Rgb888, MonoFont<'static>> {
    fn default() -> Self {
        Self {
            font: &FONT,
            font_bold: &FONT_BOLD,
            color_to_pixel: |color| color_to_rgb(color),
            dim_color: |color| dim_rgb(color),
            offset: (0, 0),
        }
    }
}

/// A default function to dim a [`Rgb888`].
pub fn dim_rgb(color: Rgb888) -> Rgb888 {
    let factor = 3;
    Rgb888::new(color.r() / factor, color.g() / factor, color.b() / factor)
}

/// A default function to convert a [`Color`] to [`Rgb888`].
///
/// You can create your own styles with your own color mapping.
pub fn color_to_rgb(color: Color) -> Rgb888 {
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
