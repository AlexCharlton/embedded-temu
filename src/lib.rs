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

mod ansi;
mod cell;
mod color;
mod console;
mod text_buffer;

pub use cell::{draw_cell_default, Cell, Flags};
pub use color::{Color, NamedColor};
pub use console::Console;
