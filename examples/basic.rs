use embedded_graphics::{pixelcolor::Rgb666, prelude::*};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay};
use embedded_temu::{Console, Style};

use std::fmt::Write;
use std::time::Instant;

const DISPLAY_SIZE: Size = Size::new(800, 600);

fn main() {
    env_logger::init();

    let mut console = Console::new(80, 24, Style::default());
    console.write_str("Hello, world!").unwrap();

    let mut display = SimulatorDisplay::<Rgb666>::new(DISPLAY_SIZE);

    let start = Instant::now();
    console.draw(&mut display).unwrap();
    println!("Draw operation took: {:?}", start.elapsed());

    let output_settings = OutputSettingsBuilder::new().build();
    let image = display.to_rgb_output_image(&output_settings);
    image.save_png("basic-output.png").unwrap();
}
