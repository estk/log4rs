
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DESERIALIZE_FOR_ConsoleAppenderConfig: () =
    {
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Deserialize for ConsoleAppenderConfig {
            fn deserialize<__D>(deserializer: __D)
             -> _serde::export::Result<ConsoleAppenderConfig, __D::Error>
             where __D: _serde::Deserializer {
                #[allow(non_camel_case_types)]
                enum __Field { __field0, __field1, }
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
                                    "target" => Ok(__Field::__field0),
                                    "encoder" => Ok(__Field::__field1),
                                    _ =>
                                    Err(_serde::de::Error::unknown_field(value,
                                                                         FIELDS)),
                                }
                            }
                            fn visit_bytes<__E>(self, value: &[u8])
                             -> _serde::export::Result<__Field, __E> where
                             __E: _serde::de::Error {
                                match value {
                                    b"target" => Ok(__Field::__field0),
                                    b"encoder" => Ok(__Field::__field1),
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
                    ConsoleAppenderConfig;
                    fn expecting(&self,
                                 formatter:
                                     &mut _serde::export::fmt::Formatter)
                     -> _serde::export::fmt::Result {
                        formatter.write_str("struct ConsoleAppenderConfig")
                    }
                    #[inline]
                    fn visit_seq<__V>(self, mut visitor: __V)
                     ->
                         _serde::export::Result<ConsoleAppenderConfig,
                                                __V::Error> where
                     __V: _serde::de::SeqVisitor {
                        let __field0 =
                            match try!(visitor . visit :: < Option <
                                       ConfigTarget > > (  )) {
                                Some(value) => { value }
                                None => {
                                    return Err(_serde::de::Error::invalid_length(0usize,
                                                                                 &"tuple of 2 elements"));
                                }
                            };
                        let __field1 =
                            match try!(visitor . visit :: < Option <
                                       EncoderConfig > > (  )) {
                                Some(value) => { value }
                                None => {
                                    return Err(_serde::de::Error::invalid_length(1usize,
                                                                                 &"tuple of 2 elements"));
                                }
                            };
                        Ok(ConsoleAppenderConfig{target: __field0,
                                                 encoder: __field1,})
                    }
                    #[inline]
                    fn visit_map<__V>(self, mut visitor: __V)
                     ->
                         _serde::export::Result<ConsoleAppenderConfig,
                                                __V::Error> where
                     __V: _serde::de::MapVisitor {
                        let mut __field0: Option<Option<ConfigTarget>> = None;
                        let mut __field1: Option<Option<EncoderConfig>> =
                            None;
                        while let Some(key) =
                                  try!(visitor . visit_key :: < __Field > (
                                       )) {
                            match key {
                                __Field::__field0 => {
                                    if __field0.is_some() {
                                        return Err(<__V::Error as
                                                       _serde::de::Error>::duplicate_field("target"));
                                    }
                                    __field0 =
                                        Some(try!(visitor . visit_value :: <
                                                  Option < ConfigTarget > > (
                                                  )));
                                }
                                __Field::__field1 => {
                                    if __field1.is_some() {
                                        return Err(<__V::Error as
                                                       _serde::de::Error>::duplicate_field("encoder"));
                                    }
                                    __field1 =
                                        Some(try!(visitor . visit_value :: <
                                                  Option < EncoderConfig > > (
                                                   )));
                                }
                            }
                        }
                        let __field0 =
                            match __field0 {
                                Some(__field0) => __field0,
                                None =>
                                try!(_serde :: de :: private :: missing_field
                                     ( "target" )),
                            };
                        let __field1 =
                            match __field1 {
                                Some(__field1) => __field1,
                                None =>
                                try!(_serde :: de :: private :: missing_field
                                     ( "encoder" )),
                            };
                        Ok(ConsoleAppenderConfig{target: __field0,
                                                 encoder: __field1,})
                    }
                }
                const FIELDS: &'static [&'static str] =
                    &["target", "encoder"];
                deserializer.deserialize_struct("ConsoleAppenderConfig",
                                                FIELDS, __Visitor)
            }
        }
    };
/// The console appender's configuration.
pub struct ConsoleAppenderConfig {
    target: Option<ConfigTarget>,
    encoder: Option<EncoderConfig>,
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DESERIALIZE_FOR_ConfigTarget: () =
    {
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Deserialize for ConfigTarget {
            fn deserialize<__D>(deserializer: __D)
             -> _serde::export::Result<ConfigTarget, __D::Error> where
             __D: _serde::Deserializer {
                #[allow(non_camel_case_types)]
                enum __Field { __field0, __field1, }
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
                            fn visit_u32<__E>(self, value: u32)
                             -> _serde::export::Result<__Field, __E> where
                             __E: _serde::de::Error {
                                match value {
                                    0u32 => Ok(__Field::__field0),
                                    1u32 => Ok(__Field::__field1),
                                    _ =>
                                    Err(_serde::de::Error::invalid_value(_serde::de::Unexpected::Unsigned(value
                                                                                                              as
                                                                                                              u64),
                                                                         &"variant index 0 <= i < 2")),
                                }
                            }
                            fn visit_str<__E>(self, value: &str)
                             -> _serde::export::Result<__Field, __E> where
                             __E: _serde::de::Error {
                                match value {
                                    "Stdout" => Ok(__Field::__field0),
                                    "Stderr" => Ok(__Field::__field1),
                                    _ =>
                                    Err(_serde::de::Error::unknown_variant(value,
                                                                           VARIANTS)),
                                }
                            }
                            fn visit_bytes<__E>(self, value: &[u8])
                             -> _serde::export::Result<__Field, __E> where
                             __E: _serde::de::Error {
                                match value {
                                    b"Stdout" => Ok(__Field::__field0),
                                    b"Stderr" => Ok(__Field::__field1),
                                    _ => {
                                        let value =
                                            &_serde::export::from_utf8_lossy(value);
                                        Err(_serde::de::Error::unknown_variant(value,
                                                                               VARIANTS))
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
                    ConfigTarget;
                    fn expecting(&self,
                                 formatter:
                                     &mut _serde::export::fmt::Formatter)
                     -> _serde::export::fmt::Result {
                        formatter.write_str("enum ConfigTarget")
                    }
                    fn visit_enum<__V>(self, visitor: __V)
                     -> _serde::export::Result<ConfigTarget, __V::Error> where
                     __V: _serde::de::EnumVisitor {
                        match try!(visitor . visit_variant (  )) {
                            (__Field::__field0, visitor) => {
                                try!(_serde :: de :: VariantVisitor ::
                                     visit_unit ( visitor ));
                                Ok(ConfigTarget::Stdout)
                            }
                            (__Field::__field1, visitor) => {
                                try!(_serde :: de :: VariantVisitor ::
                                     visit_unit ( visitor ));
                                Ok(ConfigTarget::Stderr)
                            }
                        }
                    }
                }
                const VARIANTS: &'static [&'static str] =
                    &["Stdout", "Stderr"];
                deserializer.deserialize_enum("ConfigTarget", VARIANTS,
                                              __Visitor)
            }
        }
    };
enum ConfigTarget { Stdout, Stderr, }
