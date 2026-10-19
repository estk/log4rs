#![cfg(any(feature = "file_appender", feature = "rolling_file_appender"))]

use log::{Level, LevelFilter, Log as _, Record};
use log4rs::{
    append::Append,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
    Logger,
};
use std::{fmt, sync::mpsc, thread, time::Duration};

struct LogsOnDebug<'a>(&'a Logger);

impl fmt::Debug for LogsOnDebug<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.log(
            &Record::builder()
                .level(Level::Debug)
                .args(format_args!("inner"))
                .build(),
        );
        f.write_str("value")
    }
}

fn log_recursively(appender: Box<dyn Append>) {
    let config = Config::builder()
        .appender(Appender::builder().build("file", appender))
        .build(Root::builder().appender("file").build(LevelFilter::Debug))
        .unwrap();
    let logger = Logger::new(config);
    let (sender, receiver) = mpsc::channel();
    let worker = thread::spawn(move || {
        let value = LogsOnDebug(&logger);
        logger.log(
            &Record::builder()
                .level(Level::Info)
                .args(format_args!("outer {value:?}"))
                .build(),
        );
        sender.send(()).unwrap();
    });
    receiver
        .recv_timeout(Duration::from_secs(5))
        .expect("logging from Debug::fmt deadlocked");
    worker.join().unwrap();
}

#[test]
#[cfg(feature = "file_appender")]
fn file_logging_from_debug() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("file.log");
    let appender = log4rs::append::file::FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{m}\n")))
        .build(&path)
        .unwrap();
    log_recursively(Box::new(appender));
    assert_eq!(std::fs::read(path).unwrap(), b"inner\nouter value\n");
}

#[cfg(feature = "rolling_file_appender")]
mod rolling {
    use super::*;
    use log4rs::append::rolling_file::{policy::Policy, LogFile, RollingFileAppender};
    use std::sync::{Arc, Mutex};

    #[derive(Debug)]
    struct CheckPolicy {
        pre_process: bool,
        lengths: Arc<Mutex<Vec<u64>>>,
    }

    impl Policy for CheckPolicy {
        fn process(&self, file: &mut LogFile<'_>) -> anyhow::Result<()> {
            self.lengths.lock().unwrap().push(file.len_estimate());
            Ok(())
        }

        fn is_pre_process(&self) -> bool {
            self.pre_process
        }
    }

    #[test]
    fn logging_from_debug() {
        for pre_process in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("rolling.log");
            let lengths = Arc::new(Mutex::new(Vec::new()));
            let policy = CheckPolicy {
                pre_process,
                lengths: lengths.clone(),
            };
            let appender = RollingFileAppender::builder()
                .encoder(Box::new(PatternEncoder::new("{m}\n")))
                .build(&path, Box::new(policy))
                .unwrap();
            log_recursively(Box::new(appender));
            assert_eq!(std::fs::read(path).unwrap(), b"inner\nouter value\n");
            let expected = if pre_process { vec![0, 6] } else { vec![6, 18] };
            assert_eq!(*lengths.lock().unwrap(), expected);
        }
    }

    #[derive(Debug)]
    struct RotatePolicy(bool);

    impl Policy for RotatePolicy {
        fn process(&self, file: &mut LogFile<'_>) -> anyhow::Result<()> {
            if file.len_estimate() > 0 {
                let archive = file
                    .path()
                    .with_extension(format!("{}.old", file.len_estimate()));
                file.roll();
                std::fs::rename(file.path(), archive)?;
            }
            Ok(())
        }

        fn is_pre_process(&self) -> bool {
            self.0
        }
    }

    #[test]
    fn rotation_with_recursive_logging() {
        for pre_process in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("rolling.log");
            let appender = RollingFileAppender::builder()
                .encoder(Box::new(PatternEncoder::new("{m}\n")))
                .build(&path, Box::new(RotatePolicy(pre_process)))
                .unwrap();
            log_recursively(Box::new(appender));
            assert_eq!(
                std::fs::read(path.with_extension("6.old")).unwrap(),
                b"inner\n"
            );
            let outer = if pre_process {
                path
            } else {
                assert!(!path.exists());
                path.with_extension("12.old")
            };
            assert_eq!(std::fs::read(outer).unwrap(), b"outer value\n");
        }
    }
}
