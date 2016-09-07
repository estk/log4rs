//! Implementations of the `encode::Write` trait.

#[cfg(feature = "ansi_writer")]
pub mod ansi;
#[cfg(feature = "console_writer")]
pub mod console;
#[cfg(feature = "simple_writer")]
pub mod simple;
