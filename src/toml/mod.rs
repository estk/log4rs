use std::default::Default;
use std::collections::HashMap;
use toml_parser::{self, Value};
use log::LogLevelFilter;
use std::time::Duration;

use appender::FileAppender;
use config::{self, Config};
use pattern::PatternLayout;
use Append;

mod raw;

pub trait CreateAppender: Send+'static {
    fn create_appender(&self, config: &toml_parser::Table) -> Result<Box<Append>, String>;
}

pub struct Creator {
    appenders: HashMap<String, Box<CreateAppender>>,
}

impl Default for Creator {
    fn default() -> Creator {
        let mut creator = Creator::new();
        creator.add_appender("file", Box::new(FileAppenderCreator));
        creator
    }
}

impl Creator {
    pub fn new() -> Creator {
        Creator {
            appenders: HashMap::new(),
        }
    }

    pub fn add_appender(&mut self, kind: &str, creator: Box<CreateAppender>) {
        self.appenders.insert(kind.to_string(), creator);
    }

    pub fn create_appender(&self, kind: &str, config: &toml_parser::Table)
                           -> Result<Box<Append>, String> {
        match self.appenders.get(kind) {
            Some(creator) => creator.create_appender(config),
            None => Err(format!("No creator registered for appender kind \"{}\"", kind))
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Parse(Vec<String>),
    Creation(String),
    Config(config::Error),
}

pub struct TomlConfig {
    pub refresh_rate: Option<Duration>,
    pub config: Config,
    _p: ()
}

pub fn parse(config: &str, creator: &Creator) -> Result<TomlConfig, Error> {
    let config = match raw::parse(config) {
        Ok(config) => config,
        Err(err) => return Err(Error::Parse(err)),
    };

    let raw::Config {
        refresh_rate,
        root: raw_root,
        appenders: raw_appenders,
        loggers: raw_loggers,
    } = config;

    let mut appenders = vec![];
    for (name, appender) in raw_appenders {
        let appender = match creator.create_appender(&appender.kind, &appender.config) {
            Ok(appender) => appender,
            Err(err) => return Err(Error::Creation(err)),
        };
        appenders.push(config::Appender::new(name, appender))
    }

    let root = match raw_root {
        Some(raw_root) => {
            let mut root = config::Root::new(raw_root.level);
            if let Some(appenders) = raw_root.appenders {
                root.appenders.extend(appenders.into_iter());
            }
            root
        }
        None => config::Root::new(LogLevelFilter::Debug),
    };

    let mut loggers = vec![];
    for logger in raw_loggers {
        let raw::Logger { name, level, appenders, additive } = logger;
        let mut logger = config::Logger::new(name, level);
        logger.appenders = appenders.unwrap_or(vec![]);
        logger.additive = additive.unwrap_or(true);
        loggers.push(logger);
    }

    match config::Config::new(appenders, root, loggers) {
        Ok(config) => Ok(TomlConfig {
            refresh_rate: refresh_rate,
            config: config,
            _p: (),
        }),
        Err(err) => Err(Error::Config(err))
    }
}

pub struct FileAppenderCreator;

impl CreateAppender for FileAppenderCreator {
    fn create_appender(&self, config: &toml_parser::Table) -> Result<Box<Append>, String> {
        let path = match config.get("path") {
            Some(&Value::String(ref path)) => path,
            Some(_) => return Err("`path` must be a string".to_string()),
            None => return Err("`path` is required".to_string()),
        };
        let mut appender = FileAppender::builder(path);
        match config.get("pattern") {
            Some(&Value::String(ref pattern)) => {
                appender = appender.pattern(try!(PatternLayout::new(pattern)));
            }
            Some(_) => return Err("`pattern` must be a string".to_string()),
            None => {}
        }

        match appender.build() {
            Ok(appender) => Ok(Box::new(appender)),
            Err(err) => Err(err.to_string())
        }
    }
}
