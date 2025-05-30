use embedded_graphics::{pixelcolor::Rgb666, prelude::*};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay};
use embedded_temu::{Console, Mono8BitFont, Style, color_to_rgb};
use std::time::Instant;

use std::fmt::Write;

const DISPLAY_SIZE: Size = Size::new(800, 600);
const FONT_BYTES: &[u8] = include_bytes!("./resources/RobotoMono-Regular.ttf") as &[u8];

fn main() {
    env_logger::init();
    let font = Mono8BitFont::from_font_bytes(FONT_BYTES, 16.0, Mono8BitFont::DEFAULT_GLYPHS);
    let font_bold = Mono8BitFont::from_font_bytes(FONT_BYTES, 16.0, Mono8BitFont::DEFAULT_GLYPHS);
    let style = Style::new(&font, &font_bold, color_to_rgb);

    let mut console = Console::new(80, 24, style);
    console.write_str("Hello, world!").unwrap();

    let mut display = SimulatorDisplay::<Rgb666>::new(DISPLAY_SIZE);

    // Time the draw operation
    let start = Instant::now();
    console.draw(&mut display).unwrap();
    println!("Draw operation took: {:?}", start.elapsed());

    let output_settings = OutputSettingsBuilder::new().build();
    let image = display.to_rgb_output_image(&output_settings);
    image.save_png("fontdue-output.png").unwrap();
}
