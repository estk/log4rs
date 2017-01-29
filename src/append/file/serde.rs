#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DESERIALIZE_FOR_FileAppenderConfig: () =
    {
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Deserialize for FileAppenderConfig {
            fn deserialize<__D>(deserializer: __D)
             -> _serde::export::Result<FileAppenderConfig, __D::Error> where
             __D: _serde::Deserializer {
                #[allow(non_camel_case_types)]
                enum __Field { __field0, __field1, __field2, }
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
                                    "path" => Ok(__Field::__field0),
                                    "encoder" => Ok(__Field::__field1),
                                    "append" => Ok(__Field::__field2),
                                    _ =>
                                    Err(_serde::de::Error::unknown_field(value,
                                                                         FIELDS)),
                                }
                            }
                            fn visit_bytes<__E>(self, value: &[u8])
                             -> _serde::export::Result<__Field, __E> where
                             __E: _serde::de::Error {
                                match value {
                                    b"path" => Ok(__Field::__field0),
                                    b"encoder" => Ok(__Field::__field1),
                                    b"append" => Ok(__Field::__field2),
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
                    FileAppenderConfig;
                    fn expecting(&self,
                                 formatter:
                                     &mut _serde::export::fmt::Formatter)
                     -> _serde::export::fmt::Result {
                        formatter.write_str("struct FileAppenderConfig")
                    }
                    #[inline]
                    fn visit_seq<__V>(self, mut visitor: __V)
                     -> _serde::export::Result<FileAppenderConfig, __V::Error>
                     where __V: _serde::de::SeqVisitor {
                        let __field0 =
                            match try!(visitor . visit :: < String > (  )) {
                                Some(value) => { value }
                                None => {
                                    return Err(_serde::de::Error::invalid_length(0usize,
                                                                                 &"tuple of 3 elements"));
                                }
                            };
                        let __field1 =
                            match try!(visitor . visit :: < Option <
                                       EncoderConfig > > (  )) {
                                Some(value) => { value }
                                None => {
                                    return Err(_serde::de::Error::invalid_length(1usize,
                                                                                 &"tuple of 3 elements"));
                                }
                            };
                        let __field2 =
                            match try!(visitor . visit :: < Option < bool > >
                                       (  )) {
                                Some(value) => { value }
                                None => {
                                    return Err(_serde::de::Error::invalid_length(2usize,
                                                                                 &"tuple of 3 elements"));
                                }
                            };
                        Ok(FileAppenderConfig{path: __field0,
                                              encoder: __field1,
                                              append: __field2,})
                    }
                    #[inline]
                    fn visit_map<__V>(self, mut visitor: __V)
                     -> _serde::export::Result<FileAppenderConfig, __V::Error>
                     where __V: _serde::de::MapVisitor {
                        let mut __field0: Option<String> = None;
                        let mut __field1: Option<Option<EncoderConfig>> =
                            None;
                        let mut __field2: Option<Option<bool>> = None;
                        while let Some(key) =
                                  try!(visitor . visit_key :: < __Field > (
                                       )) {
                            match key {
                                __Field::__field0 => {
                                    if __field0.is_some() {
                                        return Err(<__V::Error as
                                                       _serde::de::Error>::duplicate_field("path"));
                                    }
                                    __field0 =
                                        Some(try!(visitor . visit_value :: <
                                                  String > (  )));
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
                                __Field::__field2 => {
                                    if __field2.is_some() {
                                        return Err(<__V::Error as
                                                       _serde::de::Error>::duplicate_field("append"));
                                    }
                                    __field2 =
                                        Some(try!(visitor . visit_value :: <
                                                  Option < bool > > (  )));
                                }
                            }
                        }
                        let __field0 =
                            match __field0 {
                                Some(__field0) => __field0,
                                None =>
                                try!(_serde :: de :: private :: missing_field
                                     ( "path" )),
                            };
                        let __field1 =
                            match __field1 {
                                Some(__field1) => __field1,
                                None =>
                                try!(_serde :: de :: private :: missing_field
                                     ( "encoder" )),
                            };
                        let __field2 =
                            match __field2 {
                                Some(__field2) => __field2,
                                None =>
                                try!(_serde :: de :: private :: missing_field
                                     ( "append" )),
                            };
                        Ok(FileAppenderConfig{path: __field0,
                                              encoder: __field1,
                                              append: __field2,})
                    }
                }
                const FIELDS: &'static [&'static str] =
                    &["path", "encoder", "append"];
                deserializer.deserialize_struct("FileAppenderConfig", FIELDS,
                                                __Visitor)
            }
        }
    };
/// The file appender's configuration.
pub struct FileAppenderConfig {
    path: String,
    encoder: Option<EncoderConfig>,
    append: Option<bool>,
}
