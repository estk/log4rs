//! The dedup common handler
//!
use crate::append::Error;
use crate::encode::{Encode, Write};
use log::Record;

const REPEAT_COUNT: i32 = 1000;
/// dedup object to be used by deduping appender.
/// internals are private to dedup
#[derive(Default)]
pub struct DeDuper {
    count: i32,
    last: String,
}
#[derive(PartialEq)]
/// Used by an appender that uses dedup.
/// Indicates whether or not the current message should be output.
///
/// sample use from console appender
///         if let Some(dd) = &self.deduper {
///              if dd.lock().dedup(&mut *file, &*self.encoder, record)? == DedupResult::Skip {
///                  return Ok(());
///             }
///     ... output the message
pub enum DedupResult {
    /// skip
    Skip,
    /// write
    Write,
}
impl DeDuper {
    // emits the extra line saying 'last line repeated n times'
    fn write(
        w: &mut dyn Write,
        encoder: &dyn Encode,
        record: &Record,
        n: i32,
    ) -> Result<(), Box<dyn Error + Sync + Send>> {
        if n == 1 {
            encoder.encode(
                w,
                &Record::builder()
                    .args(format_args!("last message repeated, suppressing dups"))
                    .level(record.level())
                    .target(record.target())
                    .module_path_static(None)
                    .file_static(None)
                    .line(None)
                    .build(),
            )
        } else {
            encoder.encode(
                w,
                &Record::builder()
                    .args(format_args!("last message repeated {} times", n))
                    .level(record.level())
                    .target(record.target())
                    .module_path_static(None)
                    .file_static(None)
                    .line(None)
                    .build(),
            )
        }
    }

    /// appender calls this.
    /// If it returns Skip then appender should not write
    /// If it returns Write then the appender should write as per normal
    pub fn dedup(
        &mut self,
        w: &mut dyn Write,
        encoder: &dyn Encode,
        record: &Record,
    ) -> Result<DedupResult, Box<dyn Error + Sync + Send>> {
        let msg = format!("{}", *record.args());
        if msg == self.last {
            self.count += 1;

            // every now and then keep saying we saw lots of dups
            if self.count % REPEAT_COUNT == 0 || self.count == 1 {
                Self::write(w, encoder, record, self.count)?;
            }
            Ok(DedupResult::Skip)
        } else {
            self.last = msg;
            let svct = self.count;
            self.count = 0;
            if svct > 1 {
                Self::write(w, encoder, record, svct)?;
            }
            Ok(DedupResult::Write)
        }
    }
}
