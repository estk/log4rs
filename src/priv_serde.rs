use log::LevelFilter;
use serde::de::{self, Deserialize};
use std::fmt;

pub fn de_filter<'de, D>(d: D) -> Result<LevelFilter, D::Error>
    where D: de::Deserializer<'de>
{
    struct S(LevelFilter);

    impl<'de2> de::Deserialize<'de2> for S {
        fn deserialize<D>(d: D) -> Result<S, D::Error>
            where D: de::Deserializer<'de2>
        {
            struct V;

            impl<'de3> de::Visitor<'de3> for V {
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
