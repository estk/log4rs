#![cfg(all(
    feature = "config_parsing",
    feature = "yaml_format",
    feature = "rolling_file_appender",
    feature = "compound_policy",
    feature = "onstartup_trigger",
    feature = "size_trigger",
    feature = "time_trigger"
))]

use std::{
    fs,
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
};

use log::Record;
use log4rs::{
    append::{
        rolling_file::{
            policy::{
                compound::{roll::Roll, trigger::Trigger, CompoundPolicy},
                Policy,
            },
            LogFile, RollingFileAppender,
        },
        Append,
    },
    config::{Deserialize, Deserializers},
    encode::pattern::PatternEncoder,
};

#[derive(Clone, Debug, Default)]
struct Archives(Arc<Mutex<Vec<Vec<u8>>>>);

impl Roll for Archives {
    fn roll(&self, path: &Path) -> anyhow::Result<()> {
        self.0.lock().unwrap().push(fs::read(path)?);
        fs::remove_file(path)?;
        Ok(())
    }
}

#[derive(serde::Deserialize)]
struct RollerConfig {}

impl Deserialize for Archives {
    type Trait = dyn Roll;
    type Config = RollerConfig;

    fn deserialize(&self, _: RollerConfig, _: &Deserializers) -> anyhow::Result<Box<dyn Roll>> {
        Ok(Box::new(self.clone()))
    }
}

fn appender(path: &Path, trigger: &str, archives: &Archives) -> anyhow::Result<Box<dyn Append>> {
    let config = format!(
        "path: '{}'\nencoder:\n  pattern: '{{m}}'\npolicy:\n  trigger: {trigger}\n  roller:\n    kind: record\n",
        path.display()
    );
    let mut deserializers = Deserializers::new();
    deserializers.insert("record", archives.clone());
    deserializers.deserialize("rolling_file", serde_saphyr::from_str(&config)?)
}

fn append(appender: &dyn Append, message: &str) {
    appender
        .append(&Record::builder().args(format_args!("{message}")).build())
        .unwrap();
}

#[test]
fn startup_and_size_preserve_record_placement() {
    for triggers in [
        "[{kind: onstartup}, {kind: size, limit: 5}]",
        "[{kind: size, limit: 5}, {kind: onstartup}]",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("active.log");
        fs::write(&path, b"old").unwrap();
        let archives = Archives::default();
        let appender = appender(&path, triggers, &archives).unwrap();

        append(&*appender, "abc");
        assert_eq!(*archives.0.lock().unwrap(), vec![b"old".to_vec()]);
        assert_eq!(fs::read(&path).unwrap(), b"abc");

        append(&*appender, "def");
        assert_eq!(
            *archives.0.lock().unwrap(),
            vec![b"old".to_vec(), b"abcdef".to_vec()]
        );
        assert!(!path.exists());

        append(&*appender, "ghi");
        assert_eq!(fs::read(&path).unwrap(), b"ghi");
        assert_eq!(archives.0.lock().unwrap().len(), 2);
    }
}

#[test]
fn startup_size_and_time_roll_in_their_own_phases() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("active.log");
    fs::write(&path, b"old").unwrap();
    let archives = Archives::default();
    let appender = appender(
        &path,
        "[{kind: onstartup}, {kind: time, interval: 0 seconds}, {kind: size, limit: 5}]",
        &archives,
    )
    .unwrap();

    append(&*appender, "abcdef");
    assert_eq!(
        *archives.0.lock().unwrap(),
        vec![b"old".to_vec(), b"abcdef".to_vec()]
    );

    append(&*appender, "x");
    append(&*appender, "y");
    assert_eq!(
        *archives.0.lock().unwrap(),
        vec![b"old".to_vec(), b"abcdef".to_vec(), vec![], b"x".to_vec()]
    );
    assert_eq!(fs::read(&path).unwrap(), b"y");
}

#[test]
fn invalid_trigger_lists_are_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("active.log");
    for triggers in [
        "[]",
        "[{kind: unknown}]",
        "[{kind: size}]",
        "[{kind: onstartup, typo: true}]",
        "[{limit: 5}]",
    ] {
        assert!(
            appender(&path, triggers, &Archives::default()).is_err(),
            "{triggers}"
        );
    }
}

#[test]
fn size_rolls_after_writing_with_single_or_combined_triggers() {
    for triggers in [
        "{kind: size, limit: 5}",
        "[{kind: onstartup}, {kind: size, limit: 5}]",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("active.log");
        let archives = Archives::default();
        let appender = appender(&path, triggers, &archives).unwrap();
        append(&*appender, "abc");
        assert!(archives.0.lock().unwrap().is_empty());
        append(&*appender, "def");
        assert_eq!(*archives.0.lock().unwrap(), vec![b"abcdef".to_vec()]);
    }
}

#[derive(Debug)]
struct ExistingPolicy {
    pre: AtomicBool,
    sizes: Arc<Mutex<Vec<u64>>>,
}

impl Policy for ExistingPolicy {
    fn process(&self, log: &mut LogFile<'_>) -> anyhow::Result<()> {
        self.sizes.lock().unwrap().push(log.len_estimate());
        self.pre.fetch_xor(true, Ordering::SeqCst);
        Ok(())
    }

    fn is_pre_process(&self) -> bool {
        self.pre.load(Ordering::SeqCst)
    }
}

impl Trigger for ExistingPolicy {
    fn trigger(&self, file: &LogFile<'_>) -> anyhow::Result<bool> {
        self.sizes.lock().unwrap().push(file.len_estimate());
        self.pre.fetch_xor(true, Ordering::SeqCst);
        Ok(false)
    }

    fn is_pre_process(&self) -> bool {
        Policy::is_pre_process(self)
    }
}

#[test]
fn existing_policies_and_triggers_run_once_in_the_selected_phase() {
    for compound in [true, false] {
        for pre in [true, false] {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("active.log");
            fs::write(&path, b"old").unwrap();
            let sizes = Arc::new(Mutex::new(vec![]));
            let policy = ExistingPolicy {
                pre: AtomicBool::new(pre),
                sizes: sizes.clone(),
            };
            let policy: Box<dyn Policy> = if compound {
                Box::new(CompoundPolicy::new(
                    Box::new(policy),
                    Box::<Archives>::default(),
                ))
            } else {
                Box::new(policy)
            };
            let appender = RollingFileAppender::builder()
                .encoder(Box::new(PatternEncoder::new("{m}")))
                .build(&path, policy)
                .unwrap();
            appender
                .append(&Record::builder().args(format_args!("abc")).build())
                .unwrap();
            assert_eq!(*sizes.lock().unwrap(), vec![if pre { 3 } else { 6 }]);
            assert_eq!(fs::read(&path).unwrap(), b"oldabc");
        }
    }
}

#[derive(Debug)]
struct CountingTrigger {
    calls: Arc<AtomicUsize>,
    fail: bool,
}

#[test]
fn mixed_triggers_run_once_in_their_selected_phase() {
    for phases in [[true, false], [false, true], [true, true], [false, false]] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("active.log");
        fs::write(&path, b"old").unwrap();
        let sizes = [Arc::new(Mutex::new(vec![])), Arc::new(Mutex::new(vec![]))];
        let triggers = phases
            .into_iter()
            .zip(&sizes)
            .map(|(pre, sizes)| {
                Box::new(ExistingPolicy {
                    pre: AtomicBool::new(pre),
                    sizes: sizes.clone(),
                }) as Box<dyn Trigger>
            })
            .collect();
        let policy = CompoundPolicy::new_with_triggers(triggers, Box::<Archives>::default());
        let appender = RollingFileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{m}")))
            .build(&path, Box::new(policy))
            .unwrap();
        append(&appender, "abc");
        for (pre, sizes) in phases.into_iter().zip(&sizes) {
            assert_eq!(*sizes.lock().unwrap(), vec![if pre { 3 } else { 6 }]);
        }
        append(&appender, "def");
        for (pre, sizes) in phases.into_iter().zip(&sizes) {
            assert_eq!(
                *sizes.lock().unwrap(),
                if pre { vec![3, 6] } else { vec![6, 9] }
            );
        }
    }
}

impl Trigger for CountingTrigger {
    fn trigger(&self, _: &LogFile<'_>) -> anyhow::Result<bool> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        anyhow::ensure!(!self.fail, "trigger failed");
        Ok(true)
    }

    fn is_pre_process(&self) -> bool {
        true
    }
}

#[test]
fn evaluates_every_trigger_before_rolling_once_or_propagating_errors() {
    for fail in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("active.log");
        fs::write(&path, b"old").unwrap();
        let archives = Archives::default();
        let calls = Arc::new(AtomicUsize::new(0));
        let triggers = [false, fail]
            .into_iter()
            .map(|fail| {
                Box::new(CountingTrigger {
                    calls: calls.clone(),
                    fail,
                }) as Box<dyn Trigger>
            })
            .collect();
        let policy = CompoundPolicy::new_with_triggers(triggers, Box::new(archives.clone()));
        let appender = RollingFileAppender::builder()
            .encoder(Box::new(PatternEncoder::new("{m}")))
            .build(&path, Box::new(policy))
            .unwrap();
        let result = appender.append(&Record::builder().args(format_args!("new")).build());
        assert_eq!(calls.load(Ordering::SeqCst), 2);
        if fail {
            assert_eq!(result.unwrap_err().to_string(), "trigger failed");
            assert!(archives.0.lock().unwrap().is_empty());
            assert_eq!(fs::read(&path).unwrap(), b"old");
        } else {
            result.unwrap();
            assert_eq!(*archives.0.lock().unwrap(), vec![b"old".to_vec()]);
            assert_eq!(fs::read(&path).unwrap(), b"new");
        }
    }
}
