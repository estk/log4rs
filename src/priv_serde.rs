use log::LogLevelFilter;
use serde::de::{self, Deserialize};
use std::fmt;

pub fn de_filter<D>(d: D) -> Result<LogLevelFilter, D::Error>
    where D: de::Deserializer
{
    struct S(LogLevelFilter);

    impl de::Deserialize for S {
        fn deserialize<D>(d: D) -> Result<S, D::Error>
            where D: de::Deserializer
        {
            struct V;

            impl de::Visitor for V {
                type Value = S;

                fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                    fmt.write_str("a log level")
                }

                fn visit_str<E>(self, v: &str) -> Result<S, E>
                    where E: de::Error
                {
                    v.parse().map(S).map_err(|_| E::custom(v))
                }
            }

            d.deserialize_str(V)
        }
    }

    S::deserialize(d).map(|s| s.0)
}
