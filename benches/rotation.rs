#![feature(duration_constants)]

use std::time::{Duration, Instant};

use criterion::{criterion_group, criterion_main, Criterion};
use lazy_static::lazy_static;
use rayon::prelude::*;

const K: u64 = 1024;
const M: u64 = K * K;
const FILE_SIZE: u64 = 10 * M;
const FILE_COUNT: u32 = 10;

lazy_static! {
    static ref HANDLE: log4rs::Handle =
        log4rs::init_config(mk_config(FILE_SIZE, FILE_COUNT)).unwrap();
    static ref MSG: String = "0".repeat(80 * M as usize);
}

fn log_group(c: &mut Criterion) {
    lazy_static::initialize(&HANDLE);

    let mut group = c.benchmark_group("Log");
    group.sample_size(10);
    group.measurement_time(Duration::SECOND * 30);
    group.bench_function("log", |b| b.iter(a::write_log));
}

fn log2_group(c: &mut Criterion) {
    lazy_static::initialize(&HANDLE);

    let mut group = c.benchmark_group("Parallel Log");
    group.sample_size(10);
    group.measurement_time(Duration::SECOND * 30);
    group.bench_function("log", |b| {
        b.iter_custom(|iter| {
            let now = Instant::now();
            (0..iter).into_par_iter().for_each(|_| a::write_log());
            now.elapsed()
        })
    });
}

criterion_group!(benches, log_group, log2_group);
criterion_main!(benches);

mod a {
    pub fn write_log() {
        log::info!("{}", *super::MSG);
    }
}

fn mk_config(file_size: u64, file_count: u32) -> log4rs::config::Config {
    use log::LevelFilter;
    use log4rs::{
        append::rolling_file::{policy, RollingFileAppender},
        config::{Appender, Config, Logger, Root},
        encode::pattern::PatternEncoder,
    };
    let trigger = policy::compound::trigger::size::SizeTrigger::new(file_size);
    let roller = policy::compound::roll::fixed_window::FixedWindowRoller::builder()
        .build("logs/log.{}.gz", file_count)
        .unwrap();
    let policy = policy::compound::CompoundPolicy::new(Box::new(trigger), Box::new(roller));
    let file = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S.%3f %Z)} {l} [{t} - {T}] {m}{n}",
        )))
        .build("logs/log.log", Box::new(policy))
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
