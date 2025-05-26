//! Terminal emulator on [embedded-graphics].

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

pub use console::Console;

mod ansi;
mod cell;
mod color;
mod console;
mod text_buffer;
