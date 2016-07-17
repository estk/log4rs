#[derive(Debug, Eq, PartialEq)]
pub struct Config {
    pub refresh_rate: Option<Duration>,
    pub root: Option<Root>,
    pub appenders: HashMap<String, Appender>,
    pub loggers: HashMap<String, Logger>,
    _p: (),
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DESERIALIZE_FOR_Config: () =
    {
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::de::Deserialize for Config where

         Option<Root>: _serde::de::Deserialize,
         HashMap<String, Appender>: _serde::de::Deserialize,
         HashMap<String, Logger>: _serde::de::Deserialize,
         Option<Duration>: ::std::default::Default,
         HashMap<String, Appender>: ::std::default::Default,
         HashMap<String, Logger>: ::std::default::Default,
         (): ::std::default::Default {
            fn deserialize<__D>(deserializer: &mut __D)
             -> ::std::result::Result<Config, __D::Error> where
             __D: _serde::de::Deserializer {
                {
                    #[allow(non_camel_case_types)]
                    enum __Field {
                        __field0,
                        __field1,
                        __field2,
                        __field3,
                        __field4,
                    }
                    impl _serde::de::Deserialize for __Field {
                        #[inline]
                        fn deserialize<__D>(deserializer: &mut __D)
                         -> ::std::result::Result<__Field, __D::Error> where
                         __D: _serde::de::Deserializer {
                            struct __FieldVisitor<__D> {
                                phantom: ::std::marker::PhantomData<__D>,
                            }
                            impl <__D> _serde::de::Visitor for
                             __FieldVisitor<__D> where
                             __D: _serde::de::Deserializer {
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
                                        2usize => { Ok(__Field::__field2) }
                                        3usize => { Ok(__Field::__field3) }
                                        4usize => { Ok(__Field::__field4) }
                                        _ => {
                                            Err(_serde::de::Error::invalid_value("expected a field"))
                                        }
                                    }
                                }
                                fn visit_str<__E>(&mut self, value: &str)
                                 -> ::std::result::Result<__Field, __E> where
                                 __E: _serde::de::Error {
                                    match value {
                                        "refresh_rate" => {
                                            Ok(__Field::__field0)
                                        }
                                        "root" => { Ok(__Field::__field1) }
                                        "appenders" => {
                                            Ok(__Field::__field2)
                                        }
                                        "loggers" => { Ok(__Field::__field3) }
                                        "_p" => { Ok(__Field::__field4) }
                                        _ =>
                                        Err(_serde::de::Error::unknown_field(value)),
                                    }
                                }
                                fn visit_bytes<__E>(&mut self, value: &[u8])
                                 -> ::std::result::Result<__Field, __E> where
                                 __E: _serde::de::Error {
                                    match value {
                                        b"refresh_rate" => {
                                            Ok(__Field::__field0)
                                        }
                                        b"root" => { Ok(__Field::__field1) }
                                        b"appenders" => {
                                            Ok(__Field::__field2)
                                        }
                                        b"loggers" => {
                                            Ok(__Field::__field3)
                                        }
                                        b"_p" => { Ok(__Field::__field4) }
                                        _ => {
                                            let value =
                                                ::std::string::String::from_utf8_lossy(value);
                                            Err(_serde::de::Error::unknown_field(&value))
                                        }
                                    }
                                }
                            }
                            deserializer.deserialize_struct_field(__FieldVisitor::<__D>{phantom:
                                                                                            ::std::marker::PhantomData,})
                        }
                    }
                    struct __Visitor<__D: _serde::de::Deserializer>(::std::marker::PhantomData<__D>)
                           where Option<Root>: _serde::de::Deserialize,
                           HashMap<String, Appender>: _serde::de::Deserialize,
                           HashMap<String, Logger>: _serde::de::Deserialize,
                           Option<Duration>: ::std::default::Default,
                           HashMap<String, Appender>: ::std::default::Default,
                           HashMap<String, Logger>: ::std::default::Default,
                           (): ::std::default::Default;
                    impl <__D: _serde::de::Deserializer> _serde::de::Visitor
                     for __Visitor<__D> where
                     Option<Root>: _serde::de::Deserialize,
                     HashMap<String, Appender>: _serde::de::Deserialize,
                     HashMap<String, Logger>: _serde::de::Deserialize,
                     Option<Duration>: ::std::default::Default,
                     HashMap<String, Appender>: ::std::default::Default,
                     HashMap<String, Logger>: ::std::default::Default,
                     (): ::std::default::Default {
                        type
                        Value
                        =
                        Config;
                        #[inline]
                        fn visit_seq<__V>(&mut self, mut visitor: __V)
                         -> ::std::result::Result<Config, __V::Error> where
                         __V: _serde::de::SeqVisitor {
                            {
                                let __field0 =
                                    match {
                                              struct __SerdeDeserializeWithStruct
                                                     where
                                                     Option<Root>: _serde::de::Deserialize,
                                                     HashMap<String,
                                                             Appender>: _serde::de::Deserialize,
                                                     HashMap<String,
                                                             Logger>: _serde::de::Deserialize,
                                                     Option<Duration>: ::std::default::Default,
                                                     HashMap<String,
                                                             Appender>: ::std::default::Default,
                                                     HashMap<String,
                                                             Logger>: ::std::default::Default,
                                                     (): ::std::default::Default {
                                                  value: Option<Duration>,
                                                  phantom: ::std::marker::PhantomData<Config>,
                                              }
                                              impl _serde::de::Deserialize for
                                               __SerdeDeserializeWithStruct
                                               where
                                               Option<Root>: _serde::de::Deserialize,
                                               HashMap<String,
                                                       Appender>: _serde::de::Deserialize,
                                               HashMap<String,
                                                       Logger>: _serde::de::Deserialize,
                                               Option<Duration>: ::std::default::Default,
                                               HashMap<String,
                                                       Appender>: ::std::default::Default,
                                               HashMap<String,
                                                       Logger>: ::std::default::Default,
                                               (): ::std::default::Default {
                                                  fn deserialize<__D>(__d:
                                                                          &mut __D)
                                                   ->
                                                       ::std::result::Result<Self,
                                                                             __D::Error>
                                                   where
                                                   __D: _serde::de::Deserializer {
                                                      let value =
                                                          try!(de_duration (
                                                               __d ));
                                                      Ok(__SerdeDeserializeWithStruct{value:
                                                                                          value,
                                                                                      phantom:
                                                                                          ::std::marker::PhantomData,})
                                                  }
                                              }
                                              try!(visitor . visit :: <
                                                   __SerdeDeserializeWithStruct
                                                   > (
                                                   )).map(|wrap| wrap.value)
                                          } {
                                        Some(value) => { value }
                                        None => {
                                            try!(visitor . end (  ));
                                            return Err(_serde::de::Error::invalid_length(0usize));
                                        }
                                    };
                                let __field1 =
                                    match try!(visitor . visit :: <
                                               Option<Root> > (  )) {
                                        Some(value) => { value }
                                        None => {
                                            try!(visitor . end (  ));
                                            return Err(_serde::de::Error::invalid_length(1usize));
                                        }
                                    };
                                let __field2 =
                                    match try!(visitor . visit :: <
                                               HashMap<String, Appender> > (
                                               )) {
                                        Some(value) => { value }
                                        None => {
                                            try!(visitor . end (  ));
                                            return Err(_serde::de::Error::invalid_length(2usize));
                                        }
                                    };
                                let __field3 =
                                    match try!(visitor . visit :: <
                                               HashMap<String, Logger> > (  ))
                                        {
                                        Some(value) => { value }
                                        None => {
                                            try!(visitor . end (  ));
                                            return Err(_serde::de::Error::invalid_length(3usize));
                                        }
                                    };
                                let __field4 =
                                    ::std::default::Default::default();
                                try!(visitor . end (  ));
                                Ok(Config{refresh_rate: __field0,
                                          root: __field1,
                                          appenders: __field2,
                                          loggers: __field3,
                                          _p: __field4,})
                            }
                        }
                        #[inline]
                        fn visit_map<__V>(&mut self, mut visitor: __V)
                         -> ::std::result::Result<Config, __V::Error> where
                         __V: _serde::de::MapVisitor {
                            {
                                let mut __field0: Option<Option<Duration>> =
                                    None;
                                let mut __field1: Option<Option<Root>> = None;
                                let mut __field2:
                                        Option<HashMap<String, Appender>> =
                                    None;
                                let mut __field3:
                                        Option<HashMap<String, Logger>> =
                                    None;
                                while let Some(key) =
                                          try!(visitor . visit_key :: <
                                               __Field > (  )) {
                                    match key {
                                        __Field::__field0 => {
                                            if __field0.is_some() {
                                                return Err(<__V::Error as
                                                               _serde::de::Error>::duplicate_field("refresh_rate"));
                                            }
                                            __field0 =
                                                Some(({
                                                          struct __SerdeDeserializeWithStruct
                                                                 where
                                                                 Option<Root>: _serde::de::Deserialize,
                                                                 HashMap<String,
                                                                         Appender>: _serde::de::Deserialize,
                                                                 HashMap<String,
                                                                         Logger>: _serde::de::Deserialize,
                                                                 Option<Duration>: ::std::default::Default,
                                                                 HashMap<String,
                                                                         Appender>: ::std::default::Default,
                                                                 HashMap<String,
                                                                         Logger>: ::std::default::Default,
                                                                 (): ::std::default::Default {
                                                              value: Option<Duration>,
                                                              phantom: ::std::marker::PhantomData<Config>,
                                                          }
                                                          impl _serde::de::Deserialize
                                                           for
                                                           __SerdeDeserializeWithStruct
                                                           where
                                                           Option<Root>: _serde::de::Deserialize,
                                                           HashMap<String,
                                                                   Appender>: _serde::de::Deserialize,
                                                           HashMap<String,
                                                                   Logger>: _serde::de::Deserialize,
                                                           Option<Duration>: ::std::default::Default,
                                                           HashMap<String,
                                                                   Appender>: ::std::default::Default,
                                                           HashMap<String,
                                                                   Logger>: ::std::default::Default,
                                                           (): ::std::default::Default
                                                           {
                                                              fn deserialize<__D>(__d:
                                                                                      &mut __D)
                                                               ->
                                                                   ::std::result::Result<Self,
                                                                                         __D::Error>
                                                               where
                                                               __D: _serde::de::Deserializer {
                                                                  let value =
                                                                      try!(de_duration
                                                                           (
                                                                           __d
                                                                           ));
                                                                  Ok(__SerdeDeserializeWithStruct{value:
                                                                                                      value,
                                                                                                  phantom:
                                                                                                      ::std::marker::PhantomData,})
                                                              }
                                                          }
                                                          try!(visitor .
                                                               visit_value ::
                                                               <
                                                               __SerdeDeserializeWithStruct
                                                               > (  )).value
                                                      }));
                                        }
                                        __Field::__field1 => {
                                            if __field1.is_some() {
                                                return Err(<__V::Error as
                                                               _serde::de::Error>::duplicate_field("root"));
                                            }
                                            __field1 =
                                                Some(try!(visitor .
                                                          visit_value :: <
                                                          Option<Root> > (
                                                          )));
                                        }
                                        __Field::__field2 => {
                                            if __field2.is_some() {
                                                return Err(<__V::Error as
                                                               _serde::de::Error>::duplicate_field("appenders"));
                                            }
                                            __field2 =
                                                Some(try!(visitor .
                                                          visit_value :: <
                                                          HashMap<String, Appender>
                                                          > (  )));
                                        }
                                        __Field::__field3 => {
                                            if __field3.is_some() {
                                                return Err(<__V::Error as
                                                               _serde::de::Error>::duplicate_field("loggers"));
                                            }
                                            __field3 =
                                                Some(try!(visitor .
                                                          visit_value :: <
                                                          HashMap<String, Logger>
                                                          > (  )));
                                        }
                                        __Field::__field4 => {
                                            try!(visitor . visit_value :: <
                                                 _serde :: de :: impls ::
                                                 IgnoredAny > (  ));
                                        }
                                    }
                                }
                                try!(visitor . end (  ));
                                let __field0 =
                                    match __field0 {
                                        Some(__field0) => __field0,
                                        None =>
                                        ::std::default::Default::default(),
                                    };
                                let __field1 =
                                    match __field1 {
                                        Some(__field1) => __field1,
                                        None =>
                                        try!(visitor . missing_field ( "root"
                                             )),
                                    };
                                let __field2 =
                                    match __field2 {
                                        Some(__field2) => __field2,
                                        None =>
                                        ::std::default::Default::default(),
                                    };
                                let __field3 =
                                    match __field3 {
                                        Some(__field3) => __field3,
                                        None =>
                                        ::std::default::Default::default(),
                                    };
                                Ok(Config{refresh_rate: __field0,
                                          root: __field1,
                                          appenders: __field2,
                                          loggers: __field3,
                                          _p:
                                              ::std::default::Default::default(),})
                            }
                        }
                    }
                    const FIELDS: &'static [&'static str] =
                        &["refresh_rate", "root", "appenders", "loggers",
                          "_p"];
                    deserializer.deserialize_struct("Config", FIELDS,
                                                    __Visitor::<__D>(::std::marker::PhantomData))
                }
            }
        }
    };
#[derive(Debug, Eq, PartialEq)]
pub struct Root {
    pub level: LogLevelFilter,
    pub appenders: Vec<String>,
    _p: (),
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DESERIALIZE_FOR_Root: () =
    {
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::de::Deserialize for Root where
         Vec<String>: _serde::de::Deserialize,
         Vec<String>: ::std::default::Default, (): ::std::default::Default {
            fn deserialize<__D>(deserializer: &mut __D)
             -> ::std::result::Result<Root, __D::Error> where
             __D: _serde::de::Deserializer {
                {
                    #[allow(non_camel_case_types)]
                    enum __Field { __field0, __field1, __field2, }
                    impl _serde::de::Deserialize for __Field {
                        #[inline]
                        fn deserialize<__D>(deserializer: &mut __D)
                         -> ::std::result::Result<__Field, __D::Error> where
                         __D: _serde::de::Deserializer {
                            struct __FieldVisitor<__D> {
                                phantom: ::std::marker::PhantomData<__D>,
                            }
                            impl <__D> _serde::de::Visitor for
                             __FieldVisitor<__D> where
                             __D: _serde::de::Deserializer {
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
                                        2usize => { Ok(__Field::__field2) }
                                        _ => {
                                            Err(_serde::de::Error::invalid_value("expected a field"))
                                        }
                                    }
                                }
                                fn visit_str<__E>(&mut self, value: &str)
                                 -> ::std::result::Result<__Field, __E> where
                                 __E: _serde::de::Error {
                                    match value {
                                        "level" => { Ok(__Field::__field0) }
                                        "appenders" => {
                                            Ok(__Field::__field1)
                                        }
                                        "_p" => { Ok(__Field::__field2) }
                                        _ =>
                                        Err(_serde::de::Error::unknown_field(value)),
                                    }
                                }
                                fn visit_bytes<__E>(&mut self, value: &[u8])
                                 -> ::std::result::Result<__Field, __E> where
                                 __E: _serde::de::Error {
                                    match value {
                                        b"level" => { Ok(__Field::__field0) }
                                        b"appenders" => {
                                            Ok(__Field::__field1)
                                        }
                                        b"_p" => { Ok(__Field::__field2) }
                                        _ => {
                                            let value =
                                                ::std::string::String::from_utf8_lossy(value);
                                            Err(_serde::de::Error::unknown_field(&value))
                                        }
                                    }
                                }
                            }
                            deserializer.deserialize_struct_field(__FieldVisitor::<__D>{phantom:
                                                                                            ::std::marker::PhantomData,})
                        }
                    }
                    struct __Visitor<__D: _serde::de::Deserializer>(::std::marker::PhantomData<__D>)
                           where Vec<String>: _serde::de::Deserialize,
                           Vec<String>: ::std::default::Default,
                           (): ::std::default::Default;
                    impl <__D: _serde::de::Deserializer> _serde::de::Visitor
                     for __Visitor<__D> where
                     Vec<String>: _serde::de::Deserialize,
                     Vec<String>: ::std::default::Default,
                     (): ::std::default::Default {
                        type
                        Value
                        =
                        Root;
                        #[inline]
                        fn visit_seq<__V>(&mut self, mut visitor: __V)
                         -> ::std::result::Result<Root, __V::Error> where
                         __V: _serde::de::SeqVisitor {
                            {
                                let __field0 =
                                    match {
                                              struct __SerdeDeserializeWithStruct
                                                     where
                                                     Vec<String>: _serde::de::Deserialize,
                                                     Vec<String>: ::std::default::Default,
                                                     (): ::std::default::Default {
                                                  value: LogLevelFilter,
                                                  phantom: ::std::marker::PhantomData<Root>,
                                              }
                                              impl _serde::de::Deserialize for
                                               __SerdeDeserializeWithStruct
                                               where
                                               Vec<String>: _serde::de::Deserialize,
                                               Vec<String>: ::std::default::Default,
                                               (): ::std::default::Default {
                                                  fn deserialize<__D>(__d:
                                                                          &mut __D)
                                                   ->
                                                       ::std::result::Result<Self,
                                                                             __D::Error>
                                                   where
                                                   __D: _serde::de::Deserializer {
                                                      let value =
                                                          try!(::priv_serde::de_filter
                                                               ( __d ));
                                                      Ok(__SerdeDeserializeWithStruct{value:
                                                                                          value,
                                                                                      phantom:
                                                                                          ::std::marker::PhantomData,})
                                                  }
                                              }
                                              try!(visitor . visit :: <
                                                   __SerdeDeserializeWithStruct
                                                   > (
                                                   )).map(|wrap| wrap.value)
                                          } {
                                        Some(value) => { value }
                                        None => {
                                            try!(visitor . end (  ));
                                            return Err(_serde::de::Error::invalid_length(0usize));
                                        }
                                    };
                                let __field1 =
                                    match try!(visitor . visit :: <
                                               Vec<String> > (  )) {
                                        Some(value) => { value }
                                        None => {
                                            try!(visitor . end (  ));
                                            return Err(_serde::de::Error::invalid_length(1usize));
                                        }
                                    };
                                let __field2 =
                                    ::std::default::Default::default();
                                try!(visitor . end (  ));
                                Ok(Root{level: __field0,
                                        appenders: __field1,
                                        _p: __field2,})
                            }
                        }
                        #[inline]
                        fn visit_map<__V>(&mut self, mut visitor: __V)
                         -> ::std::result::Result<Root, __V::Error> where
                         __V: _serde::de::MapVisitor {
                            {
                                let mut __field0: Option<LogLevelFilter> =
                                    None;
                                let mut __field1: Option<Vec<String>> = None;
                                while let Some(key) =
                                          try!(visitor . visit_key :: <
                                               __Field > (  )) {
                                    match key {
                                        __Field::__field0 => {
                                            if __field0.is_some() {
                                                return Err(<__V::Error as
                                                               _serde::de::Error>::duplicate_field("level"));
                                            }
                                            __field0 =
                                                Some(({
                                                          struct __SerdeDeserializeWithStruct
                                                                 where
                                                                 Vec<String>: _serde::de::Deserialize,
                                                                 Vec<String>: ::std::default::Default,
                                                                 (): ::std::default::Default {
                                                              value: LogLevelFilter,
                                                              phantom: ::std::marker::PhantomData<Root>,
                                                          }
                                                          impl _serde::de::Deserialize
                                                           for
                                                           __SerdeDeserializeWithStruct
                                                           where
                                                           Vec<String>: _serde::de::Deserialize,
                                                           Vec<String>: ::std::default::Default,
                                                           (): ::std::default::Default
                                                           {
                                                              fn deserialize<__D>(__d:
                                                                                      &mut __D)
                                                               ->
                                                                   ::std::result::Result<Self,
                                                                                         __D::Error>
                                                               where
                                                               __D: _serde::de::Deserializer {
                                                                  let value =
                                                                      try!(::priv_serde::de_filter
                                                                           (
                                                                           __d
                                                                           ));
                                                                  Ok(__SerdeDeserializeWithStruct{value:
                                                                                                      value,
                                                                                                  phantom:
                                                                                                      ::std::marker::PhantomData,})
                                                              }
                                                          }
                                                          try!(visitor .
                                                               visit_value ::
                                                               <
                                                               __SerdeDeserializeWithStruct
                                                               > (  )).value
                                                      }));
                                        }
                                        __Field::__field1 => {
                                            if __field1.is_some() {
                                                return Err(<__V::Error as
                                                               _serde::de::Error>::duplicate_field("appenders"));
                                            }
                                            __field1 =
                                                Some(try!(visitor .
                                                          visit_value :: <
                                                          Vec<String> > (
                                                          )));
                                        }
                                        __Field::__field2 => {
                                            try!(visitor . visit_value :: <
                                                 _serde :: de :: impls ::
                                                 IgnoredAny > (  ));
                                        }
                                    }
                                }
                                try!(visitor . end (  ));
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
                                        None =>
                                        ::std::default::Default::default(),
                                    };
                                Ok(Root{level: __field0,
                                        appenders: __field1,
                                        _p:
                                            ::std::default::Default::default(),})
                            }
                        }
                    }
                    const FIELDS: &'static [&'static str] =
                        &["level", "appenders", "_p"];
                    deserializer.deserialize_struct("Root", FIELDS,
                                                    __Visitor::<__D>(::std::marker::PhantomData))
                }
            }
        }
    };
#[derive(Debug, Eq, PartialEq)]
pub struct Logger {
    pub level: LogLevelFilter,
    pub appenders: Vec<String>,
    pub additive: Option<bool>,
    _p: (),
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DESERIALIZE_FOR_Logger: () =
    {
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::de::Deserialize for Logger where
         Vec<String>: _serde::de::Deserialize,
         Option<bool>: _serde::de::Deserialize,
         Vec<String>: ::std::default::Default, (): ::std::default::Default {
            fn deserialize<__D>(deserializer: &mut __D)
             -> ::std::result::Result<Logger, __D::Error> where
             __D: _serde::de::Deserializer {
                {
                    #[allow(non_camel_case_types)]
                    enum __Field { __field0, __field1, __field2, __field3, }
                    impl _serde::de::Deserialize for __Field {
                        #[inline]
                        fn deserialize<__D>(deserializer: &mut __D)
                         -> ::std::result::Result<__Field, __D::Error> where
                         __D: _serde::de::Deserializer {
                            struct __FieldVisitor<__D> {
                                phantom: ::std::marker::PhantomData<__D>,
                            }
                            impl <__D> _serde::de::Visitor for
                             __FieldVisitor<__D> where
                             __D: _serde::de::Deserializer {
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
                                        2usize => { Ok(__Field::__field2) }
                                        3usize => { Ok(__Field::__field3) }
                                        _ => {
                                            Err(_serde::de::Error::invalid_value("expected a field"))
                                        }
                                    }
                                }
                                fn visit_str<__E>(&mut self, value: &str)
                                 -> ::std::result::Result<__Field, __E> where
                                 __E: _serde::de::Error {
                                    match value {
                                        "level" => { Ok(__Field::__field0) }
                                        "appenders" => {
                                            Ok(__Field::__field1)
                                        }
                                        "additive" => {
                                            Ok(__Field::__field2)
                                        }
                                        "_p" => { Ok(__Field::__field3) }
                                        _ =>
                                        Err(_serde::de::Error::unknown_field(value)),
                                    }
                                }
                                fn visit_bytes<__E>(&mut self, value: &[u8])
                                 -> ::std::result::Result<__Field, __E> where
                                 __E: _serde::de::Error {
                                    match value {
                                        b"level" => { Ok(__Field::__field0) }
                                        b"appenders" => {
                                            Ok(__Field::__field1)
                                        }
                                        b"additive" => {
                                            Ok(__Field::__field2)
                                        }
                                        b"_p" => { Ok(__Field::__field3) }
                                        _ => {
                                            let value =
                                                ::std::string::String::from_utf8_lossy(value);
                                            Err(_serde::de::Error::unknown_field(&value))
                                        }
                                    }
                                }
                            }
                            deserializer.deserialize_struct_field(__FieldVisitor::<__D>{phantom:
                                                                                            ::std::marker::PhantomData,})
                        }
                    }
                    struct __Visitor<__D: _serde::de::Deserializer>(::std::marker::PhantomData<__D>)
                           where Vec<String>: _serde::de::Deserialize,
                           Option<bool>: _serde::de::Deserialize,
                           Vec<String>: ::std::default::Default,
                           (): ::std::default::Default;
                    impl <__D: _serde::de::Deserializer> _serde::de::Visitor
                     for __Visitor<__D> where
                     Vec<String>: _serde::de::Deserialize,
                     Option<bool>: _serde::de::Deserialize,
                     Vec<String>: ::std::default::Default,
                     (): ::std::default::Default {
                        type
                        Value
                        =
                        Logger;
                        #[inline]
                        fn visit_seq<__V>(&mut self, mut visitor: __V)
                         -> ::std::result::Result<Logger, __V::Error> where
                         __V: _serde::de::SeqVisitor {
                            {
                                let __field0 =
                                    match {
                                              struct __SerdeDeserializeWithStruct
                                                     where
                                                     Vec<String>: _serde::de::Deserialize,
                                                     Option<bool>: _serde::de::Deserialize,
                                                     Vec<String>: ::std::default::Default,
                                                     (): ::std::default::Default {
                                                  value: LogLevelFilter,
                                                  phantom: ::std::marker::PhantomData<Logger>,
                                              }
                                              impl _serde::de::Deserialize for
                                               __SerdeDeserializeWithStruct
                                               where
                                               Vec<String>: _serde::de::Deserialize,
                                               Option<bool>: _serde::de::Deserialize,
                                               Vec<String>: ::std::default::Default,
                                               (): ::std::default::Default {
                                                  fn deserialize<__D>(__d:
                                                                          &mut __D)
                                                   ->
                                                       ::std::result::Result<Self,
                                                                             __D::Error>
                                                   where
                                                   __D: _serde::de::Deserializer {
                                                      let value =
                                                          try!(::priv_serde::de_filter
                                                               ( __d ));
                                                      Ok(__SerdeDeserializeWithStruct{value:
                                                                                          value,
                                                                                      phantom:
                                                                                          ::std::marker::PhantomData,})
                                                  }
                                              }
                                              try!(visitor . visit :: <
                                                   __SerdeDeserializeWithStruct
                                                   > (
                                                   )).map(|wrap| wrap.value)
                                          } {
                                        Some(value) => { value }
                                        None => {
                                            try!(visitor . end (  ));
                                            return Err(_serde::de::Error::invalid_length(0usize));
                                        }
                                    };
                                let __field1 =
                                    match try!(visitor . visit :: <
                                               Vec<String> > (  )) {
                                        Some(value) => { value }
                                        None => {
                                            try!(visitor . end (  ));
                                            return Err(_serde::de::Error::invalid_length(1usize));
                                        }
                                    };
                                let __field2 =
                                    match try!(visitor . visit :: <
                                               Option<bool> > (  )) {
                                        Some(value) => { value }
                                        None => {
                                            try!(visitor . end (  ));
                                            return Err(_serde::de::Error::invalid_length(2usize));
                                        }
                                    };
                                let __field3 =
                                    ::std::default::Default::default();
                                try!(visitor . end (  ));
                                Ok(Logger{level: __field0,
                                          appenders: __field1,
                                          additive: __field2,
                                          _p: __field3,})
                            }
                        }
                        #[inline]
                        fn visit_map<__V>(&mut self, mut visitor: __V)
                         -> ::std::result::Result<Logger, __V::Error> where
                         __V: _serde::de::MapVisitor {
                            {
                                let mut __field0: Option<LogLevelFilter> =
                                    None;
                                let mut __field1: Option<Vec<String>> = None;
                                let mut __field2: Option<Option<bool>> = None;
                                while let Some(key) =
                                          try!(visitor . visit_key :: <
                                               __Field > (  )) {
                                    match key {
                                        __Field::__field0 => {
                                            if __field0.is_some() {
                                                return Err(<__V::Error as
                                                               _serde::de::Error>::duplicate_field("level"));
                                            }
                                            __field0 =
                                                Some(({
                                                          struct __SerdeDeserializeWithStruct
                                                                 where
                                                                 Vec<String>: _serde::de::Deserialize,
                                                                 Option<bool>: _serde::de::Deserialize,
                                                                 Vec<String>: ::std::default::Default,
                                                                 (): ::std::default::Default {
                                                              value: LogLevelFilter,
                                                              phantom: ::std::marker::PhantomData<Logger>,
                                                          }
                                                          impl _serde::de::Deserialize
                                                           for
                                                           __SerdeDeserializeWithStruct
                                                           where
                                                           Vec<String>: _serde::de::Deserialize,
                                                           Option<bool>: _serde::de::Deserialize,
                                                           Vec<String>: ::std::default::Default,
                                                           (): ::std::default::Default
                                                           {
                                                              fn deserialize<__D>(__d:
                                                                                      &mut __D)
                                                               ->
                                                                   ::std::result::Result<Self,
                                                                                         __D::Error>
                                                               where
                                                               __D: _serde::de::Deserializer {
                                                                  let value =
                                                                      try!(::priv_serde::de_filter
                                                                           (
                                                                           __d
                                                                           ));
                                                                  Ok(__SerdeDeserializeWithStruct{value:
                                                                                                      value,
                                                                                                  phantom:
                                                                                                      ::std::marker::PhantomData,})
                                                              }
                                                          }
                                                          try!(visitor .
                                                               visit_value ::
                                                               <
                                                               __SerdeDeserializeWithStruct
                                                               > (  )).value
                                                      }));
                                        }
                                        __Field::__field1 => {
                                            if __field1.is_some() {
                                                return Err(<__V::Error as
                                                               _serde::de::Error>::duplicate_field("appenders"));
                                            }
                                            __field1 =
                                                Some(try!(visitor .
                                                          visit_value :: <
                                                          Vec<String> > (
                                                          )));
                                        }
                                        __Field::__field2 => {
                                            if __field2.is_some() {
                                                return Err(<__V::Error as
                                                               _serde::de::Error>::duplicate_field("additive"));
                                            }
                                            __field2 =
                                                Some(try!(visitor .
                                                          visit_value :: <
                                                          Option<bool> > (
                                                          )));
                                        }
                                        __Field::__field3 => {
                                            try!(visitor . visit_value :: <
                                                 _serde :: de :: impls ::
                                                 IgnoredAny > (  ));
                                        }
                                    }
                                }
                                try!(visitor . end (  ));
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
                                        None =>
                                        ::std::default::Default::default(),
                                    };
                                let __field2 =
                                    match __field2 {
                                        Some(__field2) => __field2,
                                        None =>
                                        try!(visitor . missing_field (
                                             "additive" )),
                                    };
                                Ok(Logger{level: __field0,
                                          appenders: __field1,
                                          additive: __field2,
                                          _p:
                                              ::std::default::Default::default(),})
                            }
                        }
                    }
                    const FIELDS: &'static [&'static str] =
                        &["level", "appenders", "additive", "_p"];
                    deserializer.deserialize_struct("Logger", FIELDS,
                                                    __Visitor::<__D>(::std::marker::PhantomData))
                }
            }
        }
    };
