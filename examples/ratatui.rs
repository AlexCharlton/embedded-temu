use embedded_graphics::{pixelcolor::Rgb666, prelude::*, primitives::Rectangle};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay};
use embedded_temu::{Console, EmbeddedTemuBackend, FlushableDisplay, Style};
use ratatui::Terminal;
use ratatui::widgets::Paragraph;

use std::cell::RefCell;
use std::rc::Rc;

const DISPLAY_SIZE: Size = Size::new(800, 600);

fn main() {
    env_logger::init();

    let console = Console::new(80, 24, Style::default());
    let simulator_display = Rc::new(RefCell::new(SimulatorDisplay::<Rgb666>::new(DISPLAY_SIZE)));
    let display = Display {
        display: simulator_display.clone(),
    };
    let backend = EmbeddedTemuBackend::new(console, display);
    // Create a Ratatui terminal
    let mut terminal = Terminal::new(backend).unwrap();

    // Draw to it
    terminal
        .draw(|f| {
            f.render_widget(Paragraph::new("Hello, ratatui!"), f.area());
        })
        .unwrap();

    let output_settings = OutputSettingsBuilder::new().build();
    let image = simulator_display
        .borrow()
        .to_rgb_output_image(&output_settings);
    image.save_png("ratatui-output.png").unwrap();
}

struct Display {
    display: Rc<RefCell<SimulatorDisplay<Rgb666>>>,
}

impl FlushableDisplay<<SimulatorDisplay<Rgb666> as DrawTarget>::Error, Rgb666> for Display {
    fn flush(&mut self) -> Result<(), <SimulatorDisplay<Rgb666> as DrawTarget>::Error> {
        Ok(())
    }
}

impl DrawTarget for Display {
    type Color = Rgb666;
    type Error = <SimulatorDisplay<Rgb666> as DrawTarget>::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.display.borrow_mut().draw_iter(pixels)
    }
}

impl Dimensions for Display {
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(Point::zero(), DISPLAY_SIZE)
    }
}
