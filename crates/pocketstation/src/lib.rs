//! PocketStation support for the Epok Engine.
//!
//! * [`cpu`]: an ARM7TDMI interpreter.
//! * [`hw`]: the PocketStation's memory map and devices.
//! * [`rom`], [`kernel`], [`launcher`]: Epok's own kernel ROM, system calls and
//!   start-up application, so no console firmware is needed.
//! * [`link`]: the memory card connector protocol spoken with a PlayStation.
//! * [`machine`]: everything above as one emulated PocketStation.
//! * [`card`], [`file`]: memory card images and PocketStation executables.
//! * [`asm`]: an ARM/Thumb assembler.
//! * [`plua`]: the Pocket Lua compiler and its runtime library.
//!
//! Everything here is plain Rust without dependencies so the editor, the
//! command line tools and the tests share one implementation.

pub mod asm;
pub mod card;
pub mod cpu;
pub mod file;
pub mod font;
pub mod hw;
pub mod kernel;
pub mod launcher;
pub mod link;
pub mod machine;
pub mod plua;
pub mod rom;

pub use machine::{DateTime, Machine, State, Stop};

#[cfg(test)]
mod tests;
