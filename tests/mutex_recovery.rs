#![cfg(feature = "rolling_file_appender")]

use std::{
    fs,
    panic::{catch_unwind, AssertUnwindSafe},
    sync::atomic::{AtomicBool, Ordering},
};

use log::Record;
use log4rs::{
    append::{
        rolling_file::{policy::Policy, LogFile, RollingFileAppender},
        Append,
    },
    encode::pattern::PatternEncoder,
};

#[derive(Debug)]
struct PanicOnce {
    panic: AtomicBool,
    pre_process: bool,
}

impl Policy for PanicOnce {
    fn process(&self, _: &mut LogFile<'_>) -> anyhow::Result<()> {
        assert!(!self.panic.swap(false, Ordering::SeqCst), "policy panic");
        Ok(())
    }

    fn is_pre_process(&self) -> bool {
        self.pre_process
    }
}

#[test]
fn logging_recovers_after_policy_panic() {
    for pre_process in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("active.log");
        let appender = RollingFileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{m}")))
            .build(
                &path,
                Box::new(PanicOnce {
                    panic: AtomicBool::new(true),
                    pre_process,
                }),
            )
            .unwrap();
        let first = catch_unwind(AssertUnwindSafe(|| {
            appender.append(&Record::builder().args(format_args!("first")).build())
        }));
        assert!(first.is_err());
        appender
            .append(&Record::builder().args(format_args!("second")).build())
            .unwrap();
        let expected = if pre_process { "second" } else { "firstsecond" };
        assert_eq!(fs::read_to_string(path).unwrap(), expected);
    }
}
