#![feature(duration_constants)]

use std::time::{Duration, Instant};

use lazy_static::lazy_static;

const K: u64 = 1024;
const M: u64 = K * K;
const FILE_SIZE: u64 = 100 * M;
const FILE_COUNT: u32 = 10;

lazy_static! {
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
    use std::thread;
    lazy_static::initialize(&HANDLE);

    let mut samples = stats::Unsorted::default();
    let mut mm = stats::MinMax::default();
    let mut online = stats::OnlineStats::default();
    let mut anomalies = vec![];
    let iters = 1000;
    for i in 1..iters {
        thread::sleep(Duration::from_millis(5));
        let now = Instant::now();
        a::write_log();
        let dur = now.elapsed();

        if i > 100
            && dur.as_micros() as u64 > (online.mean() + (online.stddev() * 10_f64)).round() as u64
        {
            anomalies.push(dur);
        }

        online.add(dur.as_micros());
        samples.add(dur.as_micros());
        mm.add(dur.as_micros());
    }

    if !anomalies.is_empty() {
        use humantime::format_duration;
        let min = format_duration(Duration::from_micros(*mm.min().unwrap() as u64));
        let max = format_duration(Duration::from_micros(*mm.max().unwrap() as u64));
        println!("min: {}\nmax: {}", min, max);
        let median = format_duration(Duration::from_micros(samples.median().unwrap() as u64));
        println!("median: {}", median);
        println!("anomalies: {:?}", anomalies);
    }
    assert!(anomalies.is_empty(), "There should be no log anomalies");
}

mod a {
    pub fn write_log() {
        log::info!("{}", *super::MSG);
    }
}

fn mk_config(file_size: u64, file_count: u32) -> log4rs::config::Config {
    let logdir = tempfile::tempdir().unwrap();
    let log_path = logdir.path();
    let log_pattern = log_path.join("log.log");
    let roll_pattern = format!("{}/{}", log_path.to_string_lossy(), "log.{}.gz");

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
