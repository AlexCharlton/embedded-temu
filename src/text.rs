use embedded_graphics::{
    mono_font::mapping::{GlyphMapping, StrGlyphMapping},
    prelude::*,
    primitives::Rectangle,
    text::{
        Baseline, DecorationColor, Text, TextStyle,
        renderer::{CharacterStyle, TextMetrics, TextRenderer},
    },
};
use fontdue::Font;

use alloc::vec::Vec;

use crate::cell::{Cell, Flags};
use crate::style::{ColorInterpolate, DrawCell, Style};

/// An alternative to [`embedded_graphics::mono_font::MonoFont`] that uses [`fontdue`] to render text.
pub struct Mono8BitFont {
    rasterized: Vec<u8>,
    character_size: Size,
    glyph_mapping: StrGlyphMapping<'static>,
    baseline: u32,
    glyph_bytes: usize,
}

impl Mono8BitFont {
    /// ASCII characters. TODO: Add more glyphs.
    pub const DEFAULT_GLYPHS: &'static str = "\0\u{20}\u{7f}";

    /// Get the size of the characters in the font.
    pub fn character_size(&self) -> Size {
        self.character_size
    }

    /// Create a new [`Mono8BitFont`] from the bytes of a font file and a scale (font size).
    pub fn from_font_bytes(bytes: &[u8], scale: f32, glyphs: &'static str) -> Self {
        let glyph_mapping = StrGlyphMapping::new(glyphs, '?' as usize - ' ' as usize);
        let font = Font::from_bytes(
            bytes,
            fontdue::FontSettings {
                scale,
                ..Default::default()
            },
        )
        .unwrap();
        let horizontal_line_metrics = font.horizontal_line_metrics(scale).unwrap();
        let metrics = font.metrics(' ', scale);
        let fixed_width = metrics.advance_width.ceil() as usize;
        let fixed_height = horizontal_line_metrics.new_line_size.ceil() as usize;
        let baseline = horizontal_line_metrics.ascent.round() as i32;
        debug!(
            "Creating font with line metrics: {:?}; ",
            horizontal_line_metrics
        );
        let glyph_bytes = fixed_width * fixed_height;

        // Rasterize all glyphs
        let mut rasterized = Vec::with_capacity(glyph_bytes * glyph_mapping.chars().count());
        for c in glyph_mapping.chars() {
            let (metrics, bitmap) = font.rasterize(c, scale);

            // Create a fixed-size buffer for this glyph, initialized to 0
            let mut glyph_buffer = vec![0u8; glyph_bytes];

            // Calculate how many pixels to trim from source and where to start in destination
            let (src_x_start, dst_x_start) = if metrics.xmin < 0 {
                (-metrics.xmin as usize, 0) // Start reading source after clipped pixels, write at left edge
            } else {
                (0, metrics.xmin as usize) // Read from start, offset in destination
            };

            let y_offset = baseline - metrics.ymin - metrics.height as i32;
            let (src_y_start, dst_y_start) = if y_offset < 0 {
                (-y_offset as usize, 0) // Start reading source after clipped pixels, write at top edge
            } else {
                (0, y_offset as usize) // Read from start, offset in destination
            };

            // Copy the bitmap data into the correct position in the buffer
            for y in src_y_start..metrics.height {
                for x in src_x_start..metrics.width {
                    let src_idx = y * metrics.width + x;
                    let dst_x = (x - src_x_start) + dst_x_start;
                    let dst_y = (y - src_y_start) + dst_y_start;

                    if dst_x < fixed_width && dst_y < fixed_height {
                        let dst_idx = dst_y * fixed_width + dst_x;
                        glyph_buffer[dst_idx] = bitmap[src_idx];
                    }
                }
            }

            trace!(
                "rasterized glyph: {:?}; metrics: {:?}; bitmap size: {:?}",
                c,
                metrics,
                bitmap.len()
            );

            rasterized.extend_from_slice(&glyph_buffer);
        }

        Self {
            rasterized,
            character_size: Size::new(fixed_width as u32, fixed_height as u32),
            glyph_mapping,
            baseline: baseline as u32,
            glyph_bytes,
        }
    }
}

#[derive(Clone)]
/// A style for rendering text with a [`Mono8BitFont`].
pub struct Mono8BitTextStyle<'a, C: PixelColor> {
    font: &'a Mono8BitFont,
    text_color: C,
    background_color: C,
    underline_color: DecorationColor<C>,
    strikethrough_color: DecorationColor<C>,
}

impl<'a, C: PixelColor> Mono8BitTextStyle<'a, C> {
    /// Create a new [`Mono8BitTextStyle`] with a [`Mono8BitFont`] and a text and background color.
    pub fn new(font: &'a Mono8BitFont, text_color: C, background_color: C) -> Self {
        Self {
            font,
            text_color,
            background_color,
            underline_color: DecorationColor::None,
            strikethrough_color: DecorationColor::None,
        }
    }

    /// Returns the vertical offset between the line position and the top edge of the bounding box.
    fn baseline_offset(&self, baseline: Baseline) -> i32 {
        match baseline {
            Baseline::Top => 0,
            Baseline::Bottom => self.font.character_size.height.saturating_sub(1) as i32,
            Baseline::Middle => (self.font.character_size.height.saturating_sub(1) / 2) as i32,
            Baseline::Alphabetic => self.font.baseline as i32,
        }
    }

    fn draw_decorations<D>(
        &self,
        _width: u32,
        _position: Point,
        _target: &mut D,
    ) -> Result<(), D::Error>
    where
        D: DrawTarget<Color = C>,
    {
        // if let Some(color) = self.strikethrough_color.to_color(self.text_color) {
        //     let rect = self.font.strikethrough.to_rectangle(position, width);
        //     target.fill_solid(&rect, color)?;
        // }

        // if let Some(color) = self.underline_color.to_color(self.text_color) {
        //     let rect = self.font.underline.to_rectangle(position, width);
        //     target.fill_solid(&rect, color)?;
        // }

        Ok(())
    }
}

impl<C: PixelColor + ColorInterpolate> TextRenderer for Mono8BitTextStyle<'_, C> {
    type Color = C;

    fn draw_string<D>(
        &self,
        text: &str,
        position: Point,
        baseline: Baseline,
        target: &mut D,
    ) -> Result<Point, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let mut next_position = position - Point::new(0, self.baseline_offset(baseline));

        for c in text.chars() {
            let glyph = self.font.glyph_mapping.index(c);
            let bitmap = &self.font.rasterized
                [glyph * self.font.glyph_bytes..(glyph + 1) * self.font.glyph_bytes];
            target.draw_iter(
                bitmap
                    .chunks(self.font.character_size.width as usize)
                    .enumerate()
                    .flat_map(|(row, values)| {
                        values.iter().enumerate().map(move |(col, value)| {
                            let pos = next_position + Point::new(col as i32, row as i32);
                            let color =
                                C::interpolate(self.text_color, self.background_color, *value);
                            Pixel(pos, color)
                        })
                    }),
            )?;

            next_position += Size::new(self.font.character_size.width, 0)
        }

        if next_position.x > position.x {
            let width = (next_position.x - position.x) as u32;

            self.draw_decorations(width, position, target)?;
        }

        Ok(next_position + Point::new(0, self.baseline_offset(baseline)))
    }

    fn draw_whitespace<D>(
        &self,
        width: u32,
        position: Point,
        baseline: Baseline,
        target: &mut D,
    ) -> Result<Point, D::Error>
    where
        D: DrawTarget<Color = Self::Color>,
    {
        let position = position - Point::new(0, self.baseline_offset(baseline));

        if width != 0 {
            target.fill_solid(
                &Rectangle::new(position, Size::new(width, self.font.character_size.height)),
                self.background_color,
            )?;
            self.draw_decorations(width, position, target)?;
        }

        Ok(position + Point::new(width as i32, self.baseline_offset(baseline)))
    }

    fn measure_string(&self, text: &str, position: Point, baseline: Baseline) -> TextMetrics {
        let bb_position = position - Point::new(0, self.baseline_offset(baseline));
        let bb_width = text.chars().count() as u32 * (self.font.character_size.width);

        let bb_height = if self.underline_color != DecorationColor::None {
            // self.font.underline.height + self.font.underline.offset
            0
        } else {
            self.font.character_size.height
        };

        let bb_size = Size::new(bb_width, bb_height);

        TextMetrics {
            bounding_box: Rectangle::new(bb_position, bb_size),
            next_position: position + bb_size.x_axis(),
        }
    }

    fn line_height(&self) -> u32 {
        self.font.character_size.height
    }
}

impl<C: Clone + PixelColor> CharacterStyle for Mono8BitTextStyle<'_, C> {
    type Color = C;

    fn set_text_color(&mut self, color: Option<Self::Color>) {
        if let Some(color) = color {
            self.text_color = color;
        }
    }

    fn set_background_color(&mut self, color: Option<Self::Color>) {
        if let Some(color) = color {
            self.background_color = color;
        }
    }

    fn set_underline_color(&mut self, color: DecorationColor<Self::Color>) {
        self.underline_color = color;
    }

    fn set_strikethrough_color(&mut self, color: DecorationColor<Self::Color>) {
        self.strikethrough_color = color;
    }
}

impl<'a, C> DrawCell<C> for Style<'a, C, Mono8BitFont> {
    fn draw_cell<D, P>(
        &self,
        cell: &Cell,
        row: usize,
        col: usize,
        display: &mut D,
    ) -> Result<(), <D as DrawTarget>::Error>
    where
        D: DrawTarget<Color = P>,
        P: PixelColor + From<C> + ColorInterpolate,
    {
        let mut utf8_buf = [0u8; 8];
        let s = cell.c.encode_utf8(&mut utf8_buf);
        let (fg, bg) = if cell.flags.contains(Flags::INVERSE) {
            (cell.bg, cell.fg)
        } else {
            (cell.fg, cell.bg)
        };
        let font = if cell.flags.contains(Flags::BOLD) {
            self.font_bold
        } else {
            self.font
        };
        let style = Mono8BitTextStyle::new(
            font,
            P::from(self.color_to_pixel(fg)),
            P::from(self.color_to_pixel(bg)),
        );
        if cell.flags.contains(Flags::STRIKEOUT) {
            // TODO
        }
        if cell.flags.contains(Flags::UNDERLINE) {
            // TODO
        }
        let text = Text::with_text_style(
            s,
            Point::new(
                col as i32 * self.font.character_size.width as i32 + self.offset.0 as i32,
                row as i32 * self.font.character_size.height as i32 + self.offset.1 as i32,
            ),
            style,
            TextStyle::with_baseline(Baseline::Top),
        );
        text.draw(display)?;
        Ok(())
    }
}
