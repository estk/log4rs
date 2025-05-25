//! Encoders

use derivative::Derivative;
use log::Record;
use std::{fmt, io};

#[cfg(feature = "config_parsing")]
use serde::de;
#[cfg(feature = "config_parsing")]
use serde_value::Value;
#[cfg(feature = "config_parsing")]
use std::collections::BTreeMap;

#[cfg(feature = "config_parsing")]
use crate::config::Deserializable;

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
const NEWLINE: &str = "\n";

/// A trait implemented by types that can serialize a `Record` into a
/// `Write`r.
///
/// `Encode`rs are commonly used by `Append`ers to format a log record for
/// output.
pub trait Encode: fmt::Debug + Send + Sync + 'static {
    /// Encodes the `Record` into bytes and writes them.
    fn encode(&self, w: &mut dyn Write, record: &Record) -> anyhow::Result<()>;
}

#[cfg(feature = "config_parsing")]
impl Deserializable for dyn Encode {
    fn name() -> &'static str {
        "encoder"
    }
}

/// Configuration for an encoder.
#[cfg(feature = "config_parsing")]
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct EncoderConfig {
    /// The encoder's kind.
    pub kind: String,

    /// The encoder's configuration.
    pub config: Value,
}

#[cfg(feature = "config_parsing")]
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
            kind,
            config: Value::Map(map),
        })
    }
}

/// A text or background color.
#[allow(missing_docs)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
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
#[derive(Derivative)]
#[derivative(Debug)]
#[derive(Clone, Eq, PartialEq, Hash, Default)]
pub struct Style {
    /// The text (or foreground) color.
    pub text: Option<Color>,
    /// The background color.
    pub background: Option<Color>,
    /// True if the text should have increased intensity.
    pub intense: Option<bool>,
    #[derivative(Debug = "ignore")]
    _p: (),
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

#[cfg(test)]
mod test {
    #[cfg(feature = "config_parsing")]
    use serde_test::{assert_de_tokens, assert_de_tokens_error, Token};

    #[test]
    #[cfg(feature = "config_parsing")]
    fn test_cfg_deserialize() {
        use super::*;
        use std::collections::BTreeMap;

        let pattern = "[{d(%Y-%m-%dT%H:%M:%S%.6f)} {h({l}):<5.5} {M}] {m}{n}".to_owned();

        let mut config = BTreeMap::new();
        config.insert(Value::String("pattern".to_owned()), Value::String(pattern));

        let encoder_cfg = EncoderConfig {
            kind: "pattern".to_owned(),
            config: Value::Map(config),
        };

        assert_de_tokens(
            &encoder_cfg,
            &[
                Token::Struct {
                    name: "EncoderConfig",
                    len: 2,
                },
                Token::Str("kind"),
                Token::Str("pattern"),
                Token::Str("pattern"),
                Token::Str("[{d(%Y-%m-%dT%H:%M:%S%.6f)} {h({l}):<5.5} {M}] {m}{n}"),
                Token::StructEnd,
            ],
        );

        // No pattern defined, should fail to deserializez into a map
        assert_de_tokens_error::<EncoderConfig>(
            &[
                Token::Struct {
                    name: "EncoderConfig",
                    len: 2,
                },
                Token::Str("kind"),
                Token::Str("pattern"),
                Token::Str("pattern"),
                Token::StructEnd,
            ],
            "deserialization did not expect this token: StructEnd",
        );
    }

    #[test]
    #[cfg(feature = "console_writer")]
    fn test_set_console_writer_style() {
        use super::*;
        use crate::encode::writer::console::ConsoleWriter;

        let w = match ConsoleWriter::stdout() {
            Some(w) => w,
            None => return,
        };
        let mut w = w.lock();

        assert!(w
            .set_style(
                Style::new()
                    .text(Color::Red)
                    .background(Color::Blue)
                    .intense(true),
            )
            .is_ok());

        w.set_style(&Style::new()).unwrap();
    }
}
