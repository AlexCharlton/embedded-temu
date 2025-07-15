use embedded_graphics::{pixelcolor::Rgb666, prelude::*, primitives::Rectangle};
use embedded_graphics_simulator::{OutputSettingsBuilder, SimulatorDisplay};
use embedded_temu::{
    Console, EmbeddedTemuBackend, FlushableDisplay, Mono8BitFont, RATATUI_GLYPHS, Style,
    color_to_rgb,
};
use ratatui::{
    Terminal,
    layout::Alignment,
    style::{Color, Style as RatatuiStyle, Stylize},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
};

use std::cell::RefCell;
use std::rc::Rc;

const DISPLAY_SIZE: Size = Size::new(800, 480);
const FONT_BYTES: &[u8] = include_bytes!("./resources/SourceCodePro-Regular.ttf") as &[u8];
const BOLD_FONT_BYTES: &[u8] = include_bytes!("./resources/SourceCodePro-Bold.ttf") as &[u8];

fn select_style() -> (Color, Color) {
    println!("Select a style:");
    println!("1. White on Black");
    println!("2. Black on White");
    println!("3. White on Blue");
    println!("4. Black on Blue");
    println!("5. Blue on Black");
    println!("6. Blue on White");
    println!("7. Blue on Red");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    match input.trim().parse::<u8>().unwrap_or(1) {
        2 => (Color::Black, Color::White),
        3 => (Color::White, Color::Blue),
        4 => (Color::Black, Color::Blue),
        5 => (Color::Blue, Color::Black),
        6 => (Color::Blue, Color::White),
        7 => (Color::Blue, Color::Red),
        _ => (Color::White, Color::Black), // Default
    }
}

fn main() {
    env_logger::init();

    let (fg, bg) = select_style();
    let font = Mono8BitFont::from_font_bytes(FONT_BYTES, 24.0, RATATUI_GLYPHS);
    let font_bold = Mono8BitFont::from_font_bytes(BOLD_FONT_BYTES, 24.0, RATATUI_GLYPHS);
    let mut cell_style = Style::new(&font, &font_bold, color_to_rgb);

    let cell_width = DISPLAY_SIZE.width / cell_style.font.character_size().width;
    let cell_height = DISPLAY_SIZE.height / cell_style.font.character_size().height;
    cell_style.offset = (
        (DISPLAY_SIZE.width - (cell_width * cell_style.font.character_size().width)) / 2,
        (DISPLAY_SIZE.height - (cell_height * cell_style.font.character_size().height)) / 2,
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
                    .block(
                        Block::bordered()
                            .border_set(ratatui::symbols::border::DOUBLE)
                            .padding(Padding::uniform(5)),
                    )
                    .style(ratatui::style::Style::new().fg(fg).bg(bg))
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
    const NUM_BUFFERS: usize = 2;

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
