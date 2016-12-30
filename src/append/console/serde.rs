#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DESERIALIZE_FOR_ConsoleAppenderConfig: () =
    {
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::de::Deserialize for ConsoleAppenderConfig {
            fn deserialize<__D>(deserializer: &mut __D)
             -> ::std::result::Result<ConsoleAppenderConfig, __D::Error> where
             __D: _serde::de::Deserializer {
                #[allow(non_camel_case_types)]
                enum __Field { __field0, __field1, }
                impl _serde::de::Deserialize for __Field {
                    #[inline]
                    fn deserialize<__D>(deserializer: &mut __D)
                     -> ::std::result::Result<__Field, __D::Error> where
                     __D: _serde::de::Deserializer {
                        struct __FieldVisitor;
                        impl _serde::de::Visitor for __FieldVisitor {
                            type
                            Value
                            =
                            __Field;
                            fn visit_usize<__E>(&mut self, value: usize)
                             -> ::std::result::Result<__Field, __E> where
                             __E: _serde::de::Error {
                                match value {
                                    0usize => { Ok(__Field::__field0) }
                                    1usize => { Ok(__Field::__field1) }
                                    _ =>
                                    Err(_serde::de::Error::invalid_value("expected a field")),
                                }
                            }
                            fn visit_str<__E>(&mut self, value: &str)
                             -> ::std::result::Result<__Field, __E> where
                             __E: _serde::de::Error {
                                match value {
                                    "target" => { Ok(__Field::__field0) }
                                    "encoder" => { Ok(__Field::__field1) }
                                    _ =>
                                    Err(_serde::de::Error::unknown_field(value)),
                                }
                            }
                            fn visit_bytes<__E>(&mut self, value: &[u8])
                             -> ::std::result::Result<__Field, __E> where
                             __E: _serde::de::Error {
                                match value {
                                    b"target" => { Ok(__Field::__field0) }
                                    b"encoder" => { Ok(__Field::__field1) }
                                    _ => {
                                        let value =
                                            ::std::string::String::from_utf8_lossy(value);
                                        Err(_serde::de::Error::unknown_field(&value))
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
                    #[inline]
                    fn visit_seq<__V>(&mut self, mut visitor: __V)
                     ->
                         ::std::result::Result<ConsoleAppenderConfig,
                                               __V::Error> where
                     __V: _serde::de::SeqVisitor {
                        let __field0 =
                            match try!(visitor . visit :: < Option<Target> > (
                                        )) {
                                Some(value) => { value }
                                None => {
                                    try!(visitor . end (  ));
                                    return Err(_serde::de::Error::invalid_length(0usize));
                                }
                            };
                        let __field1 =
                            match try!(visitor . visit :: <
                                       Option<EncoderConfig> > (  )) {
                                Some(value) => { value }
                                None => {
                                    try!(visitor . end (  ));
                                    return Err(_serde::de::Error::invalid_length(1usize));
                                }
                            };
                        try!(visitor . end (  ));
                        Ok(ConsoleAppenderConfig{target: __field0,
                                                 encoder: __field1,})
                    }
                    #[inline]
                    fn visit_map<__V>(&mut self, mut visitor: __V)
                     ->
                         ::std::result::Result<ConsoleAppenderConfig,
                                               __V::Error> where
                     __V: _serde::de::MapVisitor {
                        let mut __field0: Option<Option<Target>> = None;
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
                                                  Option<Target> > (  )));
                                }
                                __Field::__field1 => {
                                    if __field1.is_some() {
                                        return Err(<__V::Error as
                                                       _serde::de::Error>::duplicate_field("encoder"));
                                    }
                                    __field1 =
                                        Some(try!(visitor . visit_value :: <
                                                  Option<EncoderConfig> > (
                                                  )));
                                }
                            }
                        }
                        try!(visitor . end (  ));
                        let __field0 =
                            match __field0 {
                                Some(__field0) => __field0,
                                None =>
                                try!(visitor . missing_field ( "target" )),
                            };
                        let __field1 =
                            match __field1 {
                                Some(__field1) => __field1,
                                None =>
                                try!(visitor . missing_field ( "encoder" )),
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
    target: Option<Target>,
    encoder: Option<EncoderConfig>,
}

#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DESERIALIZE_FOR_Target: () =
    {
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::de::Deserialize for Target {
            fn deserialize<__D>(deserializer: &mut __D)
             -> ::std::result::Result<Target, __D::Error> where
             __D: _serde::de::Deserializer {
                #[allow(non_camel_case_types)]
                enum __Field { __field0, __field1, __ignore, }
                impl _serde::de::Deserialize for __Field {
                    #[inline]
                    fn deserialize<__D>(deserializer: &mut __D)
                     -> ::std::result::Result<__Field, __D::Error> where
                     __D: _serde::de::Deserializer {
                        struct __FieldVisitor;
                        impl _serde::de::Visitor for __FieldVisitor {
                            type
                            Value
                            =
                            __Field;
                            fn visit_usize<__E>(&mut self, value: usize)
                             -> ::std::result::Result<__Field, __E> where
                             __E: _serde::de::Error {
                                match value {
                                    0usize => { Ok(__Field::__field0) }
                                    1usize => { Ok(__Field::__field1) }
                                    _ =>
                                    Err(_serde::de::Error::invalid_value("expected a variant")),
                                }
                            }
                            fn visit_str<__E>(&mut self, value: &str)
                             -> ::std::result::Result<__Field, __E> where
                             __E: _serde::de::Error {
                                match value {
                                    "stdout" => { Ok(__Field::__field0) }
                                    "stderr" => { Ok(__Field::__field1) }
                                    _ =>
                                    Err(_serde::de::Error::unknown_variant(value)),
                                }
                            }
                            fn visit_bytes<__E>(&mut self, value: &[u8])
                             -> ::std::result::Result<__Field, __E> where
                             __E: _serde::de::Error {
                                match value {
                                    b"stdout" => { Ok(__Field::__field0) }
                                    b"stderr" => { Ok(__Field::__field1) }
                                    _ => {
                                        let value =
                                            ::std::string::String::from_utf8_lossy(value);
                                        Err(_serde::de::Error::unknown_variant(&value))
                                    }
                                }
                            }
                        }
                        deserializer.deserialize_struct_field(__FieldVisitor)
                    }
                }
                struct __Visitor;
                impl _serde::de::EnumVisitor for __Visitor {
                    type
                    Value
                    =
                    Target;
                    fn visit<__V>(&mut self, mut visitor: __V)
                     -> ::std::result::Result<Target, __V::Error> where
                     __V: _serde::de::VariantVisitor {
                        match try!(visitor . visit_variant (  )) {
                            __Field::__field0 => {
                                try!(visitor . visit_unit (  ));
                                Ok(Target::Stdout)
                            }
                            __Field::__field1 => {
                                try!(visitor . visit_unit (  ));
                                Ok(Target::Stderr)
                            }
                            __Field::__ignore => {
                                Err(_serde::de::Error::end_of_stream())
                            }
                        }
                    }
                }
                const VARIANTS: &'static [&'static str] =
                    &["Stdout", "Stderr"];
                deserializer.deserialize_enum("Target", VARIANTS, __Visitor)
            }
        }
    };
/// The stream to log to.
pub enum Target {

    /// Standard output.
    Stdout,

    /// Standard error.
    Stderr,
}
