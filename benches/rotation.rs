use std::{
    thread,
    time::{Duration, Instant},
};

use lazy_static::lazy_static;
use tempfile::{tempdir, TempDir};

const K: u64 = 1024;
const M: u64 = K * K;
const FILE_SIZE: u64 = 100 * M;
const FILE_COUNT: u32 = 10;

lazy_static! {
    static ref LOGDIR: TempDir = tempdir().unwrap();
    static ref HANDLE: log4rs::Handle =
        log4rs::init_config(mk_config(FILE_SIZE, FILE_COUNT)).unwrap();
    static ref MSG: String = "0".repeat(M as usize);
}
fn main() {
    bench_find_anomalies();
}

// This has been tuned on the assumption that the application will not log faster than we can gzip the files.
// This should fail with just gzip feature enabled, and succeed with features = gzip,background_rotation enabled.
fn bench_find_anomalies() {
    lazy_static::initialize(&HANDLE);

    let iters = 1000;
    let mut measurements = vec![];
    for _ in 1..iters {
        thread::sleep(Duration::from_millis(5));
        let now = Instant::now();
        a::write_log();
        let dur = now.elapsed();

        measurements.push(dur);
    }
    let stats = Stats::new(&mut measurements);
    let anomalies = stats.anomalies(&measurements);
    println!("{:#?}", stats);

    if !anomalies.is_empty() {
        println!("anomalies: {:?}", anomalies);
    }
    assert!(
        anomalies.is_empty(),
        "There should be no log anomalies: {:?}",
        anomalies
    );
}

mod a {
    pub fn write_log() {
        log::info!("{}", *super::MSG);
    }
}

fn mk_config(file_size: u64, file_count: u32) -> log4rs::config::Config {
    let log_path = LOGDIR.path();
    let log_pattern = log_path.join("log.log");

    let roll_pattern = {
        if cfg!(feature = "gzip") {
            format!("{}/{}", log_path.to_string_lossy(), "log.{}.gz")
        } else if cfg!(feature = "zstd") {
            format!("{}/{}", log_path.to_string_lossy(), "log.{}.zst")
        } else {
            format!("{}/{}", log_path.to_string_lossy(), "log.{}")
        }
    };

    use log::LevelFilter;
    use log4rs::{
        append::rolling_file::{policy, RollingFileAppender},
        config::{Appender, Config, Logger, Root},
        encode::pattern::PatternEncoder,
    };
    let trigger = policy::compound::trigger::size::SizeTrigger::new(file_size);
    let roller = policy::compound::roll::fixed_window::FixedWindowRoller::builder()
        .build(&roll_pattern, file_count)
        .unwrap();
    let policy = policy::compound::CompoundPolicy::new(Box::new(trigger), Box::new(roller));
    let file = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S.%3f %Z)} {l} [{t} - {T}] {m}{n}",
        )))
        .build(&log_pattern, Box::new(policy))
        .unwrap();

    Config::builder()
        .appender(Appender::builder().build("file", Box::new(file)))
        .logger(
            Logger::builder()
                .appender("file")
                .additive(false)
                .build("log4rs_benchmark::a", LevelFilter::Info),
        )
        .build(Root::builder().appender("file").build(LevelFilter::Info))
        .unwrap()
}

#[derive(Debug)]
struct Stats {
    min: Duration,
    max: Duration,
    median: Duration,
    mean_nanos: u128,
    variance_nanos: f64,
    stddev_nanos: f64,
}
impl Stats {
    fn new(measurements: &mut [Duration]) -> Self {
        measurements.sort();
        let (mean_nanos, variance_nanos) =
            measurements
                .iter()
                .fold((0, 0_f64), |(old_mean, old_variance), x| {
                    let nanos = x.as_nanos();
                    let size = measurements.len();

                    let mean = old_mean + (nanos - old_mean) / (size as u128);
                    let prevq = old_variance * (size as f64);
                    let variance = (prevq + ((nanos - old_mean) as f64) * (nanos - mean) as f64)
                        / (size as f64);

                    (mean, variance)
                });

        Self {
            min: measurements.first().unwrap().to_owned(),
            max: measurements.last().unwrap().to_owned(),
            median: measurements[measurements.len() / 2],
            mean_nanos,
            variance_nanos,
            stddev_nanos: variance_nanos.sqrt(),
        }
    }
    fn anomalies(&self, measurements: &[Duration]) -> Vec<Duration> {
        let mut anomalies = vec![];
        let thresh = self.mean_nanos + ((self.stddev_nanos * 50.0).round() as u128);
        for dur in measurements {
            if dur.as_nanos() as u128 > thresh {
                anomalies.push(dur.clone());
            }
        }
        anomalies
    }
}
