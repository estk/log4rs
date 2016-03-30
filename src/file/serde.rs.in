#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PrivConfig {
    refresh_rate: Option<DeDuration>,
    root: Option<Root>,
    appenders: HashMap<String, Appender>,
    loggers: HashMap<String, Logger>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PrivRoot {
    level: DeLogLevelFilter,
    #[serde(default)]
    appenders: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PrivLogger {
    level: DeLogLevelFilter,
    #[serde(default)]
    appenders: Vec<String>,
    additive: Option<bool>,
}
