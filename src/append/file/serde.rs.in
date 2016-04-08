use file::raw::Encoder;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileAppenderConfig {
    pub path: String,
    pub encoder: Option<Encoder>,
    pub append: Option<bool>,
}
