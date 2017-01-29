

#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DESERIALIZE_FOR_Config: () =
    {
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Deserialize for Config {
            fn deserialize<__D>(deserializer: __D)
             -> _serde::export::Result<Config, __D::Error> where
             __D: _serde::Deserializer {
                #[allow(non_camel_case_types)]
                enum __Field { __field0, __field1, __field2, __field3, }
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
                                    "refresh_rate" => Ok(__Field::__field0),
                                    "root" => Ok(__Field::__field1),
                                    "appenders" => Ok(__Field::__field2),
                                    "loggers" => Ok(__Field::__field3),
                                    _ =>
                                    Err(_serde::de::Error::unknown_field(value,
                                                                         FIELDS)),
                                }
                            }
                            fn visit_bytes<__E>(self, value: &[u8])
                             -> _serde::export::Result<__Field, __E> where
                             __E: _serde::de::Error {
                                match value {
                                    b"refresh_rate" => Ok(__Field::__field0),
                                    b"root" => Ok(__Field::__field1),
                                    b"appenders" => Ok(__Field::__field2),
                                    b"loggers" => Ok(__Field::__field3),
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
                    Config;
                    fn expecting(&self,
                                 formatter:
                                     &mut _serde::export::fmt::Formatter)
                     -> _serde::export::fmt::Result {
                        formatter.write_str("struct Config")
                    }
                    #[inline]
                    fn visit_seq<__V>(self, mut visitor: __V)
                     -> _serde::export::Result<Config, __V::Error> where
                     __V: _serde::de::SeqVisitor {
                        let __field0 =
                            match {
                                      struct __SerdeDeserializeWithStruct {
                                          value: Option<Duration>,
                                          phantom: _serde::export::PhantomData<Config>,
                                      }
                                      impl _serde::Deserialize for
                                       __SerdeDeserializeWithStruct {
                                          fn deserialize<__D>(__d: __D)
                                           ->
                                               _serde::export::Result<Self,
                                                                      __D::Error>
                                           where __D: _serde::Deserializer {
                                              let value =
                                                  try!(de_duration ( __d ));
                                              Ok(__SerdeDeserializeWithStruct{value:
                                                                                  value,
                                                                              phantom:
                                                                                  _serde::export::PhantomData,})
                                          }
                                      }
                                      try!(visitor . visit :: <
                                           __SerdeDeserializeWithStruct > (
                                           )).map(|wrap| wrap.value)
                                  } {
                                Some(value) => { value }
                                None => {
                                    return Err(_serde::de::Error::invalid_length(0usize,
                                                                                 &"tuple of 4 elements"));
                                }
                            };
                        let __field1 =
                            match try!(visitor . visit :: < Option < Root > >
                                       (  )) {
                                Some(value) => { value }
                                None => {
                                    return Err(_serde::de::Error::invalid_length(1usize,
                                                                                 &"tuple of 4 elements"));
                                }
                            };
                        let __field2 =
                            match try!(visitor . visit :: < HashMap < String ,
                                       Appender > > (  )) {
                                Some(value) => { value }
                                None => {
                                    return Err(_serde::de::Error::invalid_length(2usize,
                                                                                 &"tuple of 4 elements"));
                                }
                            };
                        let __field3 =
                            match try!(visitor . visit :: < HashMap < String ,
                                       Logger > > (  )) {
                                Some(value) => { value }
                                None => {
                                    return Err(_serde::de::Error::invalid_length(3usize,
                                                                                 &"tuple of 4 elements"));
                                }
                            };
                        Ok(Config{refresh_rate: __field0,
                                  root: __field1,
                                  appenders: __field2,
                                  loggers: __field3,})
                    }
                    #[inline]
                    fn visit_map<__V>(self, mut visitor: __V)
                     -> _serde::export::Result<Config, __V::Error> where
                     __V: _serde::de::MapVisitor {
                        let mut __field0: Option<Option<Duration>> = None;
                        let mut __field1: Option<Option<Root>> = None;
                        let mut __field2: Option<HashMap<String, Appender>> =
                            None;
                        let mut __field3: Option<HashMap<String, Logger>> =
                            None;
                        while let Some(key) =
                                  try!(visitor . visit_key :: < __Field > (
                                       )) {
                            match key {
                                __Field::__field0 => {
                                    if __field0.is_some() {
                                        return Err(<__V::Error as
                                                       _serde::de::Error>::duplicate_field("refresh_rate"));
                                    }
                                    __field0 =
                                        Some({
                                                 struct __SerdeDeserializeWithStruct {
                                                     value: Option<Duration>,
                                                     phantom: _serde::export::PhantomData<Config>,
                                                 }
                                                 impl _serde::Deserialize for
                                                  __SerdeDeserializeWithStruct
                                                  {
                                                     fn deserialize<__D>(__d:
                                                                             __D)
                                                      ->
                                                          _serde::export::Result<Self,
                                                                                 __D::Error>
                                                      where
                                                      __D: _serde::Deserializer {
                                                         let value =
                                                             try!(de_duration
                                                                  ( __d ));
                                                         Ok(__SerdeDeserializeWithStruct{value:
                                                                                             value,
                                                                                         phantom:
                                                                                             _serde::export::PhantomData,})
                                                     }
                                                 }
                                                 try!(visitor . visit_value ::
                                                      <
                                                      __SerdeDeserializeWithStruct
                                                      > (  )).value
                                             });
                                }
                                __Field::__field1 => {
                                    if __field1.is_some() {
                                        return Err(<__V::Error as
                                                       _serde::de::Error>::duplicate_field("root"));
                                    }
                                    __field1 =
                                        Some(try!(visitor . visit_value :: <
                                                  Option < Root > > (  )));
                                }
                                __Field::__field2 => {
                                    if __field2.is_some() {
                                        return Err(<__V::Error as
                                                       _serde::de::Error>::duplicate_field("appenders"));
                                    }
                                    __field2 =
                                        Some(try!(visitor . visit_value :: <
                                                  HashMap < String , Appender
                                                  > > (  )));
                                }
                                __Field::__field3 => {
                                    if __field3.is_some() {
                                        return Err(<__V::Error as
                                                       _serde::de::Error>::duplicate_field("loggers"));
                                    }
                                    __field3 =
                                        Some(try!(visitor . visit_value :: <
                                                  HashMap < String , Logger >
                                                  > (  )));
                                }
                            }
                        }
                        let __field0 =
                            match __field0 {
                                Some(__field0) => __field0,
                                None => _serde::export::Default::default(),
                            };
                        let __field1 =
                            match __field1 {
                                Some(__field1) => __field1,
                                None =>
                                try!(_serde :: de :: private :: missing_field
                                     ( "root" )),
                            };
                        let __field2 =
                            match __field2 {
                                Some(__field2) => __field2,
                                None => _serde::export::Default::default(),
                            };
                        let __field3 =
                            match __field3 {
                                Some(__field3) => __field3,
                                None => _serde::export::Default::default(),
                            };
                        Ok(Config{refresh_rate: __field0,
                                  root: __field1,
                                  appenders: __field2,
                                  loggers: __field3,})
                    }
                }
                const FIELDS: &'static [&'static str] =
                    &["refresh_rate", "root", "appenders", "loggers"];
                deserializer.deserialize_struct("Config", FIELDS, __Visitor)
            }
        }
    };
#[derive(Debug, Eq, PartialEq)]
pub struct Config {
    pub refresh_rate: Option<Duration>,
    pub root: Option<Root>,
    pub appenders: HashMap<String, Appender>,
    pub loggers: HashMap<String, Logger>,
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DESERIALIZE_FOR_Root: () =
    {
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Deserialize for Root {
            fn deserialize<__D>(deserializer: __D)
             -> _serde::export::Result<Root, __D::Error> where
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
                            fn visit_str<__E>(self, value: &str)
                             -> _serde::export::Result<__Field, __E> where
                             __E: _serde::de::Error {
                                match value {
                                    "level" => Ok(__Field::__field0),
                                    "appenders" => Ok(__Field::__field1),
                                    _ =>
                                    Err(_serde::de::Error::unknown_field(value,
                                                                         FIELDS)),
                                }
                            }
                            fn visit_bytes<__E>(self, value: &[u8])
                             -> _serde::export::Result<__Field, __E> where
                             __E: _serde::de::Error {
                                match value {
                                    b"level" => Ok(__Field::__field0),
                                    b"appenders" => Ok(__Field::__field1),
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
                    Root;
                    fn expecting(&self,
                                 formatter:
                                     &mut _serde::export::fmt::Formatter)
                     -> _serde::export::fmt::Result {
                        formatter.write_str("struct Root")
                    }
                    #[inline]
                    fn visit_seq<__V>(self, mut visitor: __V)
                     -> _serde::export::Result<Root, __V::Error> where
                     __V: _serde::de::SeqVisitor {
                        let __field0 =
                            match {
                                      struct __SerdeDeserializeWithStruct {
                                          value: LogLevelFilter,
                                          phantom: _serde::export::PhantomData<Root>,
                                      }
                                      impl _serde::Deserialize for
                                       __SerdeDeserializeWithStruct {
                                          fn deserialize<__D>(__d: __D)
                                           ->
                                               _serde::export::Result<Self,
                                                                      __D::Error>
                                           where __D: _serde::Deserializer {
                                              let value =
                                                  try!(:: priv_serde ::
                                                       de_filter ( __d ));
                                              Ok(__SerdeDeserializeWithStruct{value:
                                                                                  value,
                                                                              phantom:
                                                                                  _serde::export::PhantomData,})
                                          }
                                      }
                                      try!(visitor . visit :: <
                                           __SerdeDeserializeWithStruct > (
                                           )).map(|wrap| wrap.value)
                                  } {
                                Some(value) => { value }
                                None => {
                                    return Err(_serde::de::Error::invalid_length(0usize,
                                                                                 &"tuple of 2 elements"));
                                }
                            };
                        let __field1 =
                            match try!(visitor . visit :: < Vec < String > > (
                                        )) {
                                Some(value) => { value }
                                None => {
                                    return Err(_serde::de::Error::invalid_length(1usize,
                                                                                 &"tuple of 2 elements"));
                                }
                            };
                        Ok(Root{level: __field0, appenders: __field1,})
                    }
                    #[inline]
                    fn visit_map<__V>(self, mut visitor: __V)
                     -> _serde::export::Result<Root, __V::Error> where
                     __V: _serde::de::MapVisitor {
                        let mut __field0: Option<LogLevelFilter> = None;
                        let mut __field1: Option<Vec<String>> = None;
                        while let Some(key) =
                                  try!(visitor . visit_key :: < __Field > (
                                       )) {
                            match key {
                                __Field::__field0 => {
                                    if __field0.is_some() {
                                        return Err(<__V::Error as
                                                       _serde::de::Error>::duplicate_field("level"));
                                    }
                                    __field0 =
                                        Some({
                                                 struct __SerdeDeserializeWithStruct {
                                                     value: LogLevelFilter,
                                                     phantom: _serde::export::PhantomData<Root>,
                                                 }
                                                 impl _serde::Deserialize for
                                                  __SerdeDeserializeWithStruct
                                                  {
                                                     fn deserialize<__D>(__d:
                                                                             __D)
                                                      ->
                                                          _serde::export::Result<Self,
                                                                                 __D::Error>
                                                      where
                                                      __D: _serde::Deserializer {
                                                         let value =
                                                             try!(::
                                                                  priv_serde
                                                                  :: de_filter
                                                                  ( __d ));
                                                         Ok(__SerdeDeserializeWithStruct{value:
                                                                                             value,
                                                                                         phantom:
                                                                                             _serde::export::PhantomData,})
                                                     }
                                                 }
                                                 try!(visitor . visit_value ::
                                                      <
                                                      __SerdeDeserializeWithStruct
                                                      > (  )).value
                                             });
                                }
                                __Field::__field1 => {
                                    if __field1.is_some() {
                                        return Err(<__V::Error as
                                                       _serde::de::Error>::duplicate_field("appenders"));
                                    }
                                    __field1 =
                                        Some(try!(visitor . visit_value :: <
                                                  Vec < String > > (  )));
                                }
                            }
                        }
                        let __field0 =
                            match __field0 {
                                Some(__field0) => __field0,
                                None =>
                                return Err(<__V::Error as
                                               _serde::de::Error>::missing_field("level")),
                            };
                        let __field1 =
                            match __field1 {
                                Some(__field1) => __field1,
                                None => _serde::export::Default::default(),
                            };
                        Ok(Root{level: __field0, appenders: __field1,})
                    }
                }
                const FIELDS: &'static [&'static str] =
                    &["level", "appenders"];
                deserializer.deserialize_struct("Root", FIELDS, __Visitor)
            }
        }
    };
#[derive(Debug, Eq, PartialEq)]
pub struct Root {
    pub level: LogLevelFilter,
    pub appenders: Vec<String>,
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DESERIALIZE_FOR_Logger: () =
    {
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Deserialize for Logger {
            fn deserialize<__D>(deserializer: __D)
             -> _serde::export::Result<Logger, __D::Error> where
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
                                    "level" => Ok(__Field::__field0),
                                    "appenders" => Ok(__Field::__field1),
                                    "additive" => Ok(__Field::__field2),
                                    _ =>
                                    Err(_serde::de::Error::unknown_field(value,
                                                                         FIELDS)),
                                }
                            }
                            fn visit_bytes<__E>(self, value: &[u8])
                             -> _serde::export::Result<__Field, __E> where
                             __E: _serde::de::Error {
                                match value {
                                    b"level" => Ok(__Field::__field0),
                                    b"appenders" => Ok(__Field::__field1),
                                    b"additive" => Ok(__Field::__field2),
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
                    Logger;
                    fn expecting(&self,
                                 formatter:
                                     &mut _serde::export::fmt::Formatter)
                     -> _serde::export::fmt::Result {
                        formatter.write_str("struct Logger")
                    }
                    #[inline]
                    fn visit_seq<__V>(self, mut visitor: __V)
                     -> _serde::export::Result<Logger, __V::Error> where
                     __V: _serde::de::SeqVisitor {
                        let __field0 =
                            match {
                                      struct __SerdeDeserializeWithStruct {
                                          value: LogLevelFilter,
                                          phantom: _serde::export::PhantomData<Logger>,
                                      }
                                      impl _serde::Deserialize for
                                       __SerdeDeserializeWithStruct {
                                          fn deserialize<__D>(__d: __D)
                                           ->
                                               _serde::export::Result<Self,
                                                                      __D::Error>
                                           where __D: _serde::Deserializer {
                                              let value =
                                                  try!(:: priv_serde ::
                                                       de_filter ( __d ));
                                              Ok(__SerdeDeserializeWithStruct{value:
                                                                                  value,
                                                                              phantom:
                                                                                  _serde::export::PhantomData,})
                                          }
                                      }
                                      try!(visitor . visit :: <
                                           __SerdeDeserializeWithStruct > (
                                           )).map(|wrap| wrap.value)
                                  } {
                                Some(value) => { value }
                                None => {
                                    return Err(_serde::de::Error::invalid_length(0usize,
                                                                                 &"tuple of 3 elements"));
                                }
                            };
                        let __field1 =
                            match try!(visitor . visit :: < Vec < String > > (
                                        )) {
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
                        Ok(Logger{level: __field0,
                                  appenders: __field1,
                                  additive: __field2,})
                    }
                    #[inline]
                    fn visit_map<__V>(self, mut visitor: __V)
                     -> _serde::export::Result<Logger, __V::Error> where
                     __V: _serde::de::MapVisitor {
                        let mut __field0: Option<LogLevelFilter> = None;
                        let mut __field1: Option<Vec<String>> = None;
                        let mut __field2: Option<Option<bool>> = None;
                        while let Some(key) =
                                  try!(visitor . visit_key :: < __Field > (
                                       )) {
                            match key {
                                __Field::__field0 => {
                                    if __field0.is_some() {
                                        return Err(<__V::Error as
                                                       _serde::de::Error>::duplicate_field("level"));
                                    }
                                    __field0 =
                                        Some({
                                                 struct __SerdeDeserializeWithStruct {
                                                     value: LogLevelFilter,
                                                     phantom: _serde::export::PhantomData<Logger>,
                                                 }
                                                 impl _serde::Deserialize for
                                                  __SerdeDeserializeWithStruct
                                                  {
                                                     fn deserialize<__D>(__d:
                                                                             __D)
                                                      ->
                                                          _serde::export::Result<Self,
                                                                                 __D::Error>
                                                      where
                                                      __D: _serde::Deserializer {
                                                         let value =
                                                             try!(::
                                                                  priv_serde
                                                                  :: de_filter
                                                                  ( __d ));
                                                         Ok(__SerdeDeserializeWithStruct{value:
                                                                                             value,
                                                                                         phantom:
                                                                                             _serde::export::PhantomData,})
                                                     }
                                                 }
                                                 try!(visitor . visit_value ::
                                                      <
                                                      __SerdeDeserializeWithStruct
                                                      > (  )).value
                                             });
                                }
                                __Field::__field1 => {
                                    if __field1.is_some() {
                                        return Err(<__V::Error as
                                                       _serde::de::Error>::duplicate_field("appenders"));
                                    }
                                    __field1 =
                                        Some(try!(visitor . visit_value :: <
                                                  Vec < String > > (  )));
                                }
                                __Field::__field2 => {
                                    if __field2.is_some() {
                                        return Err(<__V::Error as
                                                       _serde::de::Error>::duplicate_field("additive"));
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
                                return Err(<__V::Error as
                                               _serde::de::Error>::missing_field("level")),
                            };
                        let __field1 =
                            match __field1 {
                                Some(__field1) => __field1,
                                None => _serde::export::Default::default(),
                            };
                        let __field2 =
                            match __field2 {
                                Some(__field2) => __field2,
                                None =>
                                try!(_serde :: de :: private :: missing_field
                                     ( "additive" )),
                            };
                        Ok(Logger{level: __field0,
                                  appenders: __field1,
                                  additive: __field2,})
                    }
                }
                const FIELDS: &'static [&'static str] =
                    &["level", "appenders", "additive"];
                deserializer.deserialize_struct("Logger", FIELDS, __Visitor)
            }
        }
    };
#[derive(Debug, Eq, PartialEq)]
pub struct Logger {
    pub level: LogLevelFilter,
    pub appenders: Vec<String>,
    pub additive: Option<bool>,
}
