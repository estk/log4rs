use serde::de;
use log::LogLevelFilter;
use std::time::Duration;

#[derive(PartialEq, Debug)]
pub struct DeLogLevelFilter(pub LogLevelFilter);

impl de::Deserialize for DeLogLevelFilter {
    fn deserialize<D>(d: &mut D) -> Result<DeLogLevelFilter, D::Error>
        where D: de::Deserializer
    {
        struct V;

        impl de::Visitor for V {
            type Value = DeLogLevelFilter;

            fn visit_str<E>(&mut self, v: &str) -> Result<DeLogLevelFilter, E>
                where E: de::Error
            {
                v.parse().map(DeLogLevelFilter).map_err(|_| E::invalid_value(v))
            }
        }

        d.deserialize_str(V)
    }
}

#[derive(PartialEq, Debug)]
pub struct DeDuration(pub Duration);

impl de::Deserialize for DeDuration {
    fn deserialize<D>(d: &mut D) -> Result<DeDuration, D::Error>
        where D: de::Deserializer
    {
        u64::deserialize(d).map(|r| DeDuration(Duration::from_secs(r)))
    }
}
