//! Encoders

use std::fmt;
use std::io;
use log::LogRecord;

pub mod pattern;
pub mod writer;

/// A trait implemented by types that can serialize a `LogRecord` into a
/// `Write`r.
pub trait Encode: fmt::Debug + Send + Sync + 'static {
    /// Encodes the `LogRecord` into bytes and writes them.
    fn encode(&self, w: &mut Write, record: &LogRecord) -> io::Result<()>;
}

/// A text or background color.
#[derive(Copy, Clone, Debug)]
#[allow(missing_docs)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Default,
}

/// The style applied to text output.
#[derive(Clone)]
pub struct Style {
    /// The text (or foreground) color.
    pub text: Color,
    /// The background color.
    pub background: Color,
    /// True if the text should have increased intensity.
    pub intense: bool,
    _p: (),
}

impl fmt::Debug for Style {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Style")
           .field("text", &self.text)
           .field("background", &self.background)
           .field("intense", &self.intense)
           .finish()
    }
}

impl Default for Style {
    fn default() -> Style {
        Style {
            text: Color::Default,
            background: Color::Default,
            intense: false,
            _p: (),
        }
    }
}

/// A trait for types that an `Encode`r will write to.
///
/// It extends `std::io::Write` and adds some extra functionality.
pub trait Write: io::Write {
    /// Sets the output text style, if supported.
    ///
    /// `Write`rs should ignore any parts of the `Style` they do not support.
    ///
    /// The default implementation returns `Ok(())`.
    #[allow(unused_variables)]
    fn set_style(&mut self, style: &Style) -> io::Result<()> {
        Ok(())
    }

    /// Resets the output text style, if supported.
    ///
    /// The default implementation returns `Ok(())`.
    fn reset_style(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a, W: Write> Write for &'a mut W {
    fn set_style(&mut self, style: &Style) -> io::Result<()> {
        <W as Write>::set_style(*self, style)
    }

    fn reset_style(&mut self) -> io::Result<()> {
        <W as Write>::reset_style(*self)
    }
}
