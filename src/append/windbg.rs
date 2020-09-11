//! The windbg appender.
//!
//! send output to OutputDebugStringA
//!
//!

use std::error::Error;
use std::ffi::CString;
use std::io::Write;

use crate::append::dedup::{DeDuper, DedupResult};
use crate::append::Append;
use crate::encode::EncoderConfig;
use crate::encode::{pattern::PatternEncoder, Encode};
use crate::file::Deserializers;
use log::Record;

use parking_lot::Mutex;
use serde::Deserialize;
use winapi::um::debugapi::OutputDebugStringA;

/// the windbg appender
///
pub struct WinDbgAppender {
    deduper: Option<Mutex<DeDuper>>,
    encoder: Box<dyn Encode>,
    writer: Mutex<WinDbgWriter>,
}

impl WinDbgAppender {
    /// windbg builder
    ///
    pub fn builder() -> WinDbgAppenderBuilder {
        WinDbgAppenderBuilder {
            encoder: None,
            dedup: false,
        }
    }
}

impl Append for WinDbgAppender {
    fn append(&self, record: &Record) -> Result<(), Box<dyn Error + Sync + Send>> {
        let mut wr = self.writer.lock();
        if let Some(dd) = &self.deduper {
            if dd
                .lock()
                .dedup(&mut *wr, &*self.encoder, record)?
                == DedupResult::Skip
            {
                return Ok(());
            }
        }

        self.encoder.encode(&mut *wr, record)?;
        wr.flush()?;
        Ok(())
    }
    fn flush(&self) {}
}

impl std::fmt::Debug for WinDbgAppender {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("WinDbgAppender").finish()
    }
}

#[derive(Default, Debug)]
struct WinDbgWriter {
    buf: Vec<u8>,
}

impl std::io::Write for WinDbgWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        let mut t = vec![];
        t.extend_from_slice(&self.buf);
        unsafe {
            let str = CString::new(t).unwrap();
            OutputDebugStringA(str.as_ptr());
        }
        self.buf.clear();
        Ok(())
    }
}

impl crate::encode::Write for WinDbgWriter {}

#[derive(Deserialize)]
/// windbg config
/// encode is standard option
/// dedup asks for message dedupping
///
pub struct WinDbgConfig {
    encoder: Option<EncoderConfig>,
    dedup: Option<bool>,
}

/// windbg deseriazlier
/// move along, nothing intersing here
pub struct WinDbgAppenderDeserializer;

impl crate::file::Deserialize for WinDbgAppenderDeserializer {
    type Trait = dyn Append;
    type Config = WinDbgConfig;

    fn deserialize(
        &self,
        config: WinDbgConfig,
        deserializers: &Deserializers,
    ) -> Result<Box<dyn Append>, Box<dyn Error + Sync + Send>> {
        let mut appender = WinDbgAppender::builder();
        if let Some(dedup) = config.dedup {
            appender = appender.dedup(dedup);
        }
        if let Some(encoder) = config.encoder {
            appender = appender.encoder(deserializers.deserialize(&encoder.kind, encoder.config)?);
        }
        Ok(Box::new(appender.build()?))
    }
}
/// A builder for `windbgappender`s.
///
pub struct WinDbgAppenderBuilder {
    encoder: Option<Box<dyn Encode>>,
    dedup: bool,
}

/// windbg builder
/// only option is dedup (boolean)
impl WinDbgAppenderBuilder {
    /// dedup or not
    pub fn dedup(mut self, dedup: bool) -> WinDbgAppenderBuilder {
        self.dedup = dedup;
        self
    }
    /// encoder
    pub fn encoder(mut self, encoder: Box<dyn Encode>) -> WinDbgAppenderBuilder {
        self.encoder = Some(encoder);
        self
    }

    /// build
    pub fn build(self) -> std::io::Result<WinDbgAppender> {
        let deduper = {
            if self.dedup {
                Some(Mutex::new(DeDuper::default()))
            } else {
                None
            }
        };
        Ok(WinDbgAppender {
            deduper: deduper,
            encoder: self
                .encoder
                .unwrap_or_else(|| Box::new(PatternEncoder::default())),
            writer: Mutex::new(WinDbgWriter::default()),
        })
    }
}
