use embedded_graphics::{pixelcolor::Rgb666, prelude::*, primitives::Rectangle};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay};
use embedded_temu::{Console, EmbeddedTemuBackend, FlushableDisplay, Style};
use ratatui::{
    Terminal,
    layout::Alignment,
    style::{Style as RatatuiStyle, Stylize},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
};

use std::cell::RefCell;
use std::rc::Rc;

const DISPLAY_SIZE: Size = Size::new(800, 600);

fn main() {
    env_logger::init();

    let mut cell_style = Style::default();
    let cell_width = DISPLAY_SIZE.width / cell_style.font.character_size.width;
    let cell_height = DISPLAY_SIZE.height / cell_style.font.character_size.height;
    cell_style.offset = (
        (DISPLAY_SIZE.width - (cell_width * cell_style.font.character_size.width)) / 2,
        (DISPLAY_SIZE.height - (cell_height * cell_style.font.character_size.height)) / 2,
    );

    let console = Console::new(cell_width as usize, cell_height as usize, cell_style);
    let simulator_display = Rc::new(RefCell::new(SimulatorDisplay::<Rgb666>::new(DISPLAY_SIZE)));
    let display = Display {
        display: simulator_display.clone(),
    };
    let backend = EmbeddedTemuBackend::new(console, display);
    // Create a Ratatui terminal
    let mut terminal = Terminal::new(backend).unwrap();

    // Draw to it
    let text = vec![
        Line::from(vec![
            Span::raw("Hello, "),
            Span::styled("Ratatui", RatatuiStyle::new().green().italic()),
            "!".into(),
        ]),
        Line::from("I love you".red()),
    ];
    terminal
        .draw(|f| {
            f.render_widget(
                Paragraph::new(text)
                    .block(Block::bordered().padding(Padding::uniform(5)))
                    .alignment(Alignment::Center),
                f.area(),
            );
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
