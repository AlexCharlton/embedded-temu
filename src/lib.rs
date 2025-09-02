//! A terminal emulator for [`embedded_graphics`].

#![no_std]
#![deny(missing_docs)]

#[macro_use]
extern crate alloc;

#[cfg(feature = "log")]
#[macro_use]
extern crate log;
#[cfg(not(feature = "log"))]
#[macro_use]
mod log;

#[cfg(feature = "ratatui-backend")]
mod ratatui_backend;
#[cfg(feature = "ratatui-backend")]
pub use ratatui_backend::*;

#[cfg(feature = "fontdue")]
mod text;
#[cfg(feature = "fontdue")]
pub use text::{Mono8BitFont, Mono8BitTextStyle};

mod ansi;
mod cell;
mod cell_buffer;
mod color;
mod console;
mod style;

pub use color::{Color, NamedColor};
pub use console::Console;
pub use style::{ColorInterpolate, Style, color_to_rgb, dim_rgb};

/// Utility functions
pub mod util {
    pub use super::style::interpolate_8bit_values;
}
