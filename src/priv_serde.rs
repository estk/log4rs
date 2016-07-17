use humantime;
use log::LogLevelFilter;
use serde::de::{self, Deserialize};
use std::time::Duration;

pub fn de_filter<D>(d: &mut D) -> Result<LogLevelFilter, D::Error>
    where D: de::Deserializer
{
    struct S(LogLevelFilter);

    impl de::Deserialize for S {
        fn deserialize<D>(d: &mut D) -> Result<S, D::Error>
            where D: de::Deserializer
        {
            struct V;

            impl de::Visitor for V {
                type Value = S;

                fn visit_str<E>(&mut self, v: &str) -> Result<S, E>
                    where E: de::Error
                {
                    v.parse().map(S).map_err(|_| E::invalid_value(v))
                }
            }

            d.deserialize_str(V)
        }
    }

    S::deserialize(d).map(|s| s.0)
}

#[derive(PartialEq, Debug)]
pub struct DeDuration(pub Duration);

impl de::Deserialize for DeDuration {
    fn deserialize<D>(d: &mut D) -> Result<DeDuration, D::Error>
        where D: de::Deserializer
    {
        struct V;

        impl de::Visitor for V {
            type Value = DeDuration;

            fn visit_str<E>(&mut self, v: &str) -> Result<DeDuration, E>
                where E: de::Error
            {
                match humantime::parse_duration(v) {
                    Ok(d) => Ok(DeDuration(d)),
                    Err(e) => Err(E::invalid_value(&e.to_string())),
                }
            }

            // for back-compat
            fn visit_i64<E>(&mut self, v: i64) -> Result<DeDuration, E>
                where E: de::Error
            {
                if v < 0 {
                    return Err(E::invalid_value("Duration cannot be negative"));
                }
                Ok(DeDuration(Duration::from_secs(v as u64)))
            }

            fn visit_u64<E>(&mut self, v: u64) -> Result<DeDuration, E>
                where E: de::Error
            {
                // for back-compat
                Ok(DeDuration(Duration::from_secs(v)))
            }
        }

        d.deserialize(V)
    }
}
