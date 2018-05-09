//! Encoders

use log::Record;
#[cfg(feature = "file")]
use serde::de;
#[cfg(feature = "file")]
use serde_value::Value;
#[cfg(feature = "file")]
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::io;

#[cfg(feature = "file")]
use file::Deserializable;
use record::ExtendedRecord;

#[cfg(feature = "json_encoder")]
pub mod json;
#[cfg(feature = "pattern_encoder")]
pub mod pattern;
pub mod writer;

#[allow(dead_code)]
#[cfg(windows)]
const NEWLINE: &'static str = "\r\n";
#[allow(dead_code)]
#[cfg(not(windows))]
const NEWLINE: &'static str = "\n";

/// A trait implemented by types that can serialize a `Record` into a
/// `Write`r.
///
/// `Encode`rs are commonly used by `Append`ers to format a log record for
/// output.
pub trait Encode: fmt::Debug + Send + Sync + 'static {
    /// Encodes the `Record` into bytes and writes them.
    fn encode(&self, w: &mut Write, record: &ExtendedRecord) -> Result<(), Box<Error + Sync + Send>>;
}

#[cfg(feature = "file")]
impl Deserializable for Encode {
    fn name() -> &'static str {
        "encoder"
    }
}

/// Configuration for an encoder.
#[cfg(feature = "file")]
pub struct EncoderConfig {
    /// The encoder's kind.
    pub kind: String,

    /// The encoder's configuration.
    pub config: Value,
}

#[cfg(feature = "file")]
impl<'de> de::Deserialize<'de> for EncoderConfig {
    fn deserialize<D>(d: D) -> Result<EncoderConfig, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let mut map = BTreeMap::<Value, Value>::deserialize(d)?;

        let kind = match map.remove(&Value::String("kind".to_owned())) {
            Some(kind) => kind.deserialize_into().map_err(|e| e.to_error())?,
            None => "pattern".to_owned(),
        };

        Ok(EncoderConfig {
            kind: kind,
            config: Value::Map(map),
        })
    }
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
}

/// The style applied to text output.
///
/// Any fields set to `None` will be set to their default format, as defined
/// by the `Write`r.
#[derive(Clone, Default)]
pub struct Style {
    /// The text (or foreground) color.
    pub text: Option<Color>,
    /// The background color.
    pub background: Option<Color>,
    /// True if the text should have increased intensity.
    pub intense: Option<bool>,
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

impl Style {
    /// Returns a `Style` with all fields set to their defaults.
    pub fn new() -> Style {
        Style::default()
    }

    /// Sets the text color.
    pub fn text(&mut self, text: Color) -> &mut Style {
        self.text = Some(text);
        self
    }

    /// Sets the background color.
    pub fn background(&mut self, background: Color) -> &mut Style {
        self.background = Some(background);
        self
    }

    /// Sets the text intensity.
    pub fn intense(&mut self, intense: bool) -> &mut Style {
        self.intense = Some(intense);
        self
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
    /// The default implementation returns `Ok(())`. Implementations that do
    /// not support styling should do this as well.
    #[allow(unused_variables)]
    fn set_style(&mut self, style: &Style) -> io::Result<()> {
        Ok(())
    }
}

impl<'a, W: Write + ?Sized> Write for &'a mut W {
    fn set_style(&mut self, style: &Style) -> io::Result<()> {
        <W as Write>::set_style(*self, style)
    }
}
