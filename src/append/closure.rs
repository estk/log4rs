//! The closure appender.
//!
//! send output to an arbitray closure
//!
//!
//! 
use std::any::Any;
use std::io::Write;
use std::cell::RefCell;
use std::error::Error;

use crate::append::dedup::{DeDuper, DedupResult};
use crate::append::Append;
use crate::encode::EncoderConfig;
use crate::encode::{pattern::PatternEncoder, Encode};
use crate::file::Deserializers;
use log::Record;
use parking_lot::Mutex;
use serde::Deserialize;

/// the windbg appender
///
pub struct ClosureAppender {
    deduper: Option<Mutex<DeDuper>>,
    encoder: Box<dyn Encode>,
    writer: Mutex<RefCell<ClosureWriter>>,
}

type LogFunc = Box<dyn Fn(Vec<u8>) + Send + Sync>;
impl ClosureAppender {
    /// closure builder
    ///
    pub fn builder() -> ClosureAppenderBuilder {
        ClosureAppenderBuilder {
            encoder: None,
            dedup: false,
            func: None,
        }
    }
    /// programmatically set the closue if using a config file
    /// 
    pub fn closure(conf: &crate::config::Config, name: &str, func: LogFunc) {
        let apps = conf.appenders();
        if let Some(c) = apps.iter().find(|&x| x.name() == name) {
            if let Some(cla) = c.appender().as_any().downcast_ref::<ClosureAppender>() {
                let wr = cla.writer.lock();
                wr.borrow_mut().func.replace(func);
            };
        };
    }
}

impl Append for ClosureAppender {
    fn append(&self, record: &Record) -> Result<(), Box<dyn Error + Sync + Send>> {
        let l = self.writer.lock();
        let mut wr = l.borrow_mut();

        if let Some(dd) = &self.deduper {
            if dd.lock().dedup(&mut *wr, &*self.encoder, record)? == DedupResult::Skip {
                return Ok(());
            }
        }

        self.encoder.encode(&mut *wr, record)?;
        wr.flush()?;
        Ok(())
    }
    fn flush(&self) {}
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::fmt::Debug for ClosureAppender {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("ClosureAppender").finish()
    }
}

struct ClosureWriter {
    buf: Vec<u8>,
    func: Option<LogFunc>,
}
impl std::fmt::Debug for ClosureWriter {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("ClosureWriter").finish()
    }
}
impl std::io::Write for ClosureWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        let mut t = vec![];
        t.extend_from_slice(&self.buf);
        if let Some(ref f) = self.func {
            f(t);
        }
        self.buf.clear();
        Ok(())
    }
}

impl crate::encode::Write for ClosureWriter {}

#[derive(Deserialize)]
/// closure config
/// encode is standard option
/// dedup asks for message dedupping
///
pub struct ClosureConfig {
    encoder: Option<EncoderConfig>,
    dedup: Option<bool>,
}

/// closure deseriazlier
/// move along, nothing intersing here
pub struct ClosureAppenderDeserializer;

impl crate::file::Deserialize for ClosureAppenderDeserializer {
    type Trait = dyn Append;
    type Config = ClosureConfig;

    fn deserialize(
        &self,
        config: ClosureConfig,
        deserializers: &Deserializers,
    ) -> Result<Box<dyn Append>, Box<dyn Error + Sync + Send>> {
        let mut appender = ClosureAppender::builder();
        if let Some(dedup) = config.dedup {
            appender = appender.dedup(dedup);
        }
        if let Some(encoder) = config.encoder {
            appender = appender.encoder(deserializers.deserialize(&encoder.kind, encoder.config)?);
        }
        Ok(Box::new(appender.build()?))
    }
}
/// A builder for `closureappender`s.
///
pub struct ClosureAppenderBuilder {
    encoder: Option<Box<dyn Encode>>,
    dedup: bool,
    func: Option<LogFunc>,
}

/// closure builder
/// must set closure
/// only option is dedup (boolean)
impl ClosureAppenderBuilder {
    /// dedup or not
    pub fn dedup(mut self, dedup: bool) -> ClosureAppenderBuilder {
        self.dedup = dedup;
        self
    }
    /// encoder
    pub fn encoder(mut self, encoder: Box<dyn Encode>) -> ClosureAppenderBuilder {
        self.encoder = Some(encoder);
        self
    }
    /// closure
    pub fn closure(mut self, func: LogFunc) -> ClosureAppenderBuilder {
        self.func = Some(func);
        self
    }
    /// build
    pub fn build(self) -> std::io::Result<ClosureAppender> {
        let deduper = {
            if self.dedup {
                Some(Mutex::new(DeDuper::default()))
            } else {
                None
            }
        };
        Ok(ClosureAppender {
            deduper: deduper,
            encoder: self
                .encoder
                .unwrap_or_else(|| Box::new(PatternEncoder::default())),
            writer: Mutex::new(RefCell::new(ClosureWriter {
                buf: Vec::with_capacity(100),
                func: self.func,
            })),
        })
    }
}
