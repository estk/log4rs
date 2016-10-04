use log::LogLevelFilter;
use serde::de::{self, Deserialize};

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
