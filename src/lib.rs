//! A terminal emulator for [`embedded_graphics`].

#![no_std]
#![deny(unsafe_code)]
#![deny(warnings)]
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

mod ansi;
mod cell;
mod color;
mod console;
mod style;
mod text_buffer;

pub use color::{Color, NamedColor};
pub use console::Console;
pub use style::Style;
