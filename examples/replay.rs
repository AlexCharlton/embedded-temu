use std::fmt::Write;
use std::time::Instant;

use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay};
use embedded_temu::{Console, Style};

const DISPLAY_SIZE: Size = Size::new(800, 480);

fn main() {
    env_logger::init();
    let mut display = SimulatorDisplay::<Rgb888>::new(DISPLAY_SIZE);

    let mut args = std::env::args_os();
    args.next(); // skip program name
    let fname = args
        .next()
        .expect("Usage: replay <ANSI_ESCAPE_SEQUENCE_FILE>");
    let input = std::fs::read_to_string(fname.clone()).unwrap();
    let decoded = input.replace("\\x1b", "\x1b");
    println!("Read {} bytes from {:?}", decoded.len(), fname);

    let mut console = Console::new(80, 24, Style::default());
    let time = Instant::now();
    console.write_str(&decoded).unwrap();
    println!("Render time: {:?}", time.elapsed());

    console.draw(&mut display).unwrap();
    let output_settings = OutputSettingsBuilder::new().build();
    let image = display.to_rgb_output_image(&output_settings);
    image.save_png("replay-output.png").unwrap();
}
