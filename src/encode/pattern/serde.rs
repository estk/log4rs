#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DESERIALIZE_FOR_PatternEncoderConfig: () =
    {
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Deserialize for PatternEncoderConfig {
            fn deserialize<__D>(deserializer: __D)
             -> _serde::export::Result<PatternEncoderConfig, __D::Error> where
             __D: _serde::Deserializer {
                #[allow(non_camel_case_types)]
                enum __Field { __field0, }
                impl _serde::Deserialize for __Field {
                    #[inline]
                    fn deserialize<__D>(deserializer: __D)
                     -> _serde::export::Result<__Field, __D::Error> where
                     __D: _serde::Deserializer {
                        struct __FieldVisitor;
                        impl _serde::de::Visitor for __FieldVisitor {
                            type
                            Value
                            =
                            __Field;
                            fn expecting(&self,
                                         formatter:
                                             &mut _serde::export::fmt::Formatter)
                             -> _serde::export::fmt::Result {
                                formatter.write_str("field name")
                            }
                            fn visit_str<__E>(self, value: &str)
                             -> _serde::export::Result<__Field, __E> where
                             __E: _serde::de::Error {
                                match value {
                                    "pattern" => Ok(__Field::__field0),
                                    _ =>
                                    Err(_serde::de::Error::unknown_field(value,
                                                                         FIELDS)),
                                }
                            }
                            fn visit_bytes<__E>(self, value: &[u8])
                             -> _serde::export::Result<__Field, __E> where
                             __E: _serde::de::Error {
                                match value {
                                    b"pattern" => Ok(__Field::__field0),
                                    _ => {
                                        let value =
                                            &_serde::export::from_utf8_lossy(value);
                                        Err(_serde::de::Error::unknown_field(value,
                                                                             FIELDS))
                                    }
                                }
                            }
                        }
                        deserializer.deserialize_struct_field(__FieldVisitor)
                    }
                }
                struct __Visitor;
                impl _serde::de::Visitor for __Visitor {
                    type
                    Value
                    =
                    PatternEncoderConfig;
                    fn expecting(&self,
                                 formatter:
                                     &mut _serde::export::fmt::Formatter)
                     -> _serde::export::fmt::Result {
                        formatter.write_str("struct PatternEncoderConfig")
                    }
                    #[inline]
                    fn visit_seq<__V>(self, mut visitor: __V)
                     ->
                         _serde::export::Result<PatternEncoderConfig,
                                                __V::Error> where
                     __V: _serde::de::SeqVisitor {
                        let __field0 =
                            match try!(visitor . visit :: < Option < String >
                                       > (  )) {
                                Some(value) => { value }
                                None => {
                                    return Err(_serde::de::Error::invalid_length(0usize,
                                                                                 &"tuple of 1 elements"));
                                }
                            };
                        Ok(PatternEncoderConfig{pattern: __field0,})
                    }
                    #[inline]
                    fn visit_map<__V>(self, mut visitor: __V)
                     ->
                         _serde::export::Result<PatternEncoderConfig,
                                                __V::Error> where
                     __V: _serde::de::MapVisitor {
                        let mut __field0: Option<Option<String>> = None;
                        while let Some(key) =
                                  try!(visitor . visit_key :: < __Field > (
                                       )) {
                            match key {
                                __Field::__field0 => {
                                    if __field0.is_some() {
                                        return Err(<__V::Error as
                                                       _serde::de::Error>::duplicate_field("pattern"));
                                    }
                                    __field0 =
                                        Some(try!(visitor . visit_value :: <
                                                  Option < String > > (  )));
                                }
                            }
                        }
                        let __field0 =
                            match __field0 {
                                Some(__field0) => __field0,
                                None =>
                                try!(_serde :: de :: private :: missing_field
                                     ( "pattern" )),
                            };
                        Ok(PatternEncoderConfig{pattern: __field0,})
                    }
                }
                const FIELDS: &'static [&'static str] = &["pattern"];
                deserializer.deserialize_struct("PatternEncoderConfig",
                                                FIELDS, __Visitor)
            }
        }
    };
/// The pattern encoder's configuration.
pub struct PatternEncoderConfig {
    pattern: Option<String>,
}
