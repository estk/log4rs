use serde::de;

#[derive(PartialEq, Eq, Debug, Default)]
pub struct Undeserializable;

impl de::Deserialize for Undeserializable {
    fn deserialize<D>(_: &mut D) -> Result<Undeserializable, D::Error> where D: de::Deserializer {
        Err(de::Error::invalid_value("field name is reserved"))
    }
}
