use embedded_graphics::{pixelcolor::Rgb666, prelude::*};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay};
use embedded_term::{draw_cell_default, Console};

use std::fmt::Write;

const DISPLAY_SIZE: Size = Size::new(800, 600);

fn main() {
    env_logger::init();

    let mut console = Console::new(80, 24);
    console.write_str("Hello, world!").unwrap();

    let mut display = SimulatorDisplay::<Rgb666>::new(DISPLAY_SIZE);
    console.draw(&mut display, draw_cell_default).unwrap();

    let output_settings = OutputSettingsBuilder::new().build();
    let image = display.to_rgb_output_image(&output_settings);
    image.save_png("rterm-output.png").unwrap();
}
