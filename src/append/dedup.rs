//! The dedup common handler
//!
use crate::append::Error;
use crate::encode::{Encode, Write};
use log::Record;

const REPEAT_COUNT: i32 = 1000;

#[derive(Default)]
pub(crate) struct DeDuper {
    count: i32,
    last: String,
}
#[derive(PartialEq)]

pub(crate) enum DedupResult {
    /// skip
    Skip,
    /// write
    Write,
}
impl DeDuper {
    // emits the extra line saying 'last line repeated n times'
    fn say(
        w: &mut dyn Write,
        encoder: &dyn Encode,
        record: &Record,
        n: i32,
    ) -> Result<(), Box<dyn Error + Sync + Send>> {
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

    // appender calls this.
    // If it retunrs Skip then appender should not write
    // if Write then the appender should write as per normal

    pub(crate) fn dedup(
        &mut self,
        w: &mut dyn Write,
        encoder: &dyn Encode,
        record: &Record,
    ) -> Result<DedupResult, Box<dyn Error + Sync + Send>> {
        let msg = format!("{}", *record.args());
        if msg == self.last {
            self.count += 1;

            // every now and then keep saying we saw lots of dups
            if self.count % REPEAT_COUNT == 0 {
                Self::say(w, encoder, record, self.count)?;
            }
            Ok(DedupResult::Skip)
        } else {
            self.last = msg;
            let svct = self.count;
            self.count = 0;
            if svct > 0 {
                Self::say(w, encoder, record, svct)?;
            }
            Ok(DedupResult::Write)
        }
    }
}
