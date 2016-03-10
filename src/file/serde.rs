#[derive_Debug]
#[derive_PartialEq]
pub struct Config {
    pub refresh_rate: Option<DeDuration>,
    pub root: Option<Root>,
    pub appenders: HashMap<String, Appender>,
    pub loggers: HashMap<String, Logger>,
}
impl ::serde::de::Deserialize for Config {
    fn deserialize<__D>(deserializer: &mut __D)
     -> ::std::result::Result<Config, __D::Error> where
     __D: ::serde::de::Deserializer {
        {
            #[allow(non_camel_case_types)]
            enum __Field { __field0, __field1, __field2, __field3, __ignore, }
            impl ::serde::de::Deserialize for __Field {
                #[inline]
                fn deserialize<D>(deserializer: &mut D)
                 -> ::std::result::Result<__Field, D::Error> where
                 D: ::serde::de::Deserializer {
                    use std::marker::PhantomData;
                    struct __FieldVisitor<D> {
                        phantom: PhantomData<D>,
                    }
                    impl <__D> ::serde::de::Visitor for __FieldVisitor<__D>
                     where __D: ::serde::de::Deserializer {
                        type
                        Value
                        =
                        __Field;
                        fn visit_usize<E>(&mut self, value: usize)
                         -> ::std::result::Result<__Field, E> where
                         E: ::serde::de::Error {
                            match value {
                                0usize => { Ok(__Field::__field0) }
                                1usize => { Ok(__Field::__field1) }
                                2usize => { Ok(__Field::__field2) }
                                3usize => { Ok(__Field::__field3) }
                                _ => Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<E>(&mut self, value: &str)
                         -> ::std::result::Result<__Field, E> where
                         E: ::serde::de::Error {
                            match value {
                                "refresh_rate" => { Ok(__Field::__field0) }
                                "root" => { Ok(__Field::__field1) }
                                "appenders" => { Ok(__Field::__field2) }
                                "loggers" => { Ok(__Field::__field3) }
                                _ => Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<E>(&mut self, value: &[u8])
                         -> ::std::result::Result<__Field, E> where
                         E: ::serde::de::Error {
                            match value {
                                b"refresh_rate" => { Ok(__Field::__field0) }
                                b"root" => { Ok(__Field::__field1) }
                                b"appenders" => { Ok(__Field::__field2) }
                                b"loggers" => { Ok(__Field::__field3) }
                                _ => Ok(__Field::__ignore),
                            }
                        }
                    }
                    deserializer.deserialize_struct_field(__FieldVisitor::<D>{phantom:
                                                                                  PhantomData,})
                }
            }
            struct __Visitor<__D: ::serde::de::Deserializer>(::std::marker::PhantomData<__D>);
            impl <__D: 





                  ::serde::de::Deserializer> ::serde::de::Visitor for
             __Visitor<__D> {
                type
                Value
                =
                Config;
                #[inline]
                fn visit_seq<__V>(&mut self, mut visitor: __V)
                 -> ::std::result::Result<Config, __V::Error> where
                 __V: ::serde::de::SeqVisitor {
                    {
                        let __field0 =
                            match try!(visitor . visit (  )) {
                                Some(value) => { value }
                                None => {
                                    return Err(::serde::de::Error::end_of_stream());
                                }
                            };
                        let __field1 =
                            match try!(visitor . visit (  )) {
                                Some(value) => { value }
                                None => {
                                    return Err(::serde::de::Error::end_of_stream());
                                }
                            };
                        let __field2 =
                            match try!(visitor . visit (  )) {
                                Some(value) => { value }
                                None => {
                                    return Err(::serde::de::Error::end_of_stream());
                                }
                            };
                        let __field3 =
                            match try!(visitor . visit (  )) {
                                Some(value) => { value }
                                None => {
                                    return Err(::serde::de::Error::end_of_stream());
                                }
                            };
                        try!(visitor . end (  ));
                        Ok(Config{refresh_rate: __field0,
                                  root: __field1,
                                  appenders: __field2,
                                  loggers: __field3,})
                    }
                }
                #[inline]
                fn visit_map<__V>(&mut self, mut visitor: __V)
                 -> ::std::result::Result<Config, __V::Error> where
                 __V: ::serde::de::MapVisitor {
                    {
                        let mut __field0 = None;
                        let mut __field1 = None;
                        let mut __field2 = None;
                        let mut __field3 = None;
                        while let Some(key) = try!(visitor . visit_key (  )) {
                            match key {
                                __Field::__field0 => {
                                    __field0 =
                                        Some(try!(visitor.visit_value()));
                                }
                                __Field::__field1 => {
                                    __field1 =
                                        Some(try!(visitor.visit_value()));
                                }
                                __Field::__field2 => {
                                    __field2 =
                                        Some(try!(visitor.visit_value()));
                                }
                                __Field::__field3 => {
                                    __field3 =
                                        Some(try!(visitor.visit_value()));
                                }
                                _ => {
                                    try!(visitor . visit_value:: < :: serde::
                                         de:: impls:: IgnoredAny > (  ));
                                }
                            }
                        }
                        let __field0 =
                            match __field0 {
                                Some(__field0) => __field0,
                                None =>
                                match visitor.missing_field("refresh_rate") {
                                    ::std::result::Result::Ok(value) => value,
                                    ::std::result::Result::Err(value) =>
                                    return ::std::result::Result::Err(value),
                                },
                            };
                        let __field1 =
                            match __field1 {
                                Some(__field1) => __field1,
                                None =>
                                match visitor.missing_field("root") {
                                    ::std::result::Result::Ok(value) => value,
                                    ::std::result::Result::Err(value) =>
                                    return ::std::result::Result::Err(value),
                                },
                            };
                        let __field2 =
                            match __field2 {
                                Some(__field2) => __field2,
                                None =>
                                match visitor.missing_field("appenders") {
                                    ::std::result::Result::Ok(value) => value,
                                    ::std::result::Result::Err(value) =>
                                    return ::std::result::Result::Err(value),
                                },
                            };
                        let __field3 =
                            match __field3 {
                                Some(__field3) => __field3,
                                None =>
                                match visitor.missing_field("loggers") {
                                    ::std::result::Result::Ok(value) => value,
                                    ::std::result::Result::Err(value) =>
                                    return ::std::result::Result::Err(value),
                                },
                            };
                        try!(visitor . end (  ));
                        Ok(Config{refresh_rate: __field0,
                                  root: __field1,
                                  appenders: __field2,
                                  loggers: __field3,})
                    }
                }
            }
            const FIELDS: &'static [&'static str] =
                &["refresh_rate", "root", "appenders", "loggers"];
            deserializer.deserialize_struct("Config", FIELDS,
                                            __Visitor::<__D>(::std::marker::PhantomData))
        }
    }
}
#[derive_Debug]
#[derive_PartialEq]
pub struct Root {
    pub level: DeLogLevelFilter,
    pub appenders: Vec<String>,
}
impl ::serde::de::Deserialize for Root {
    fn deserialize<__D>(deserializer: &mut __D)
     -> ::std::result::Result<Root, __D::Error> where
     __D: ::serde::de::Deserializer {
        {
            #[allow(non_camel_case_types)]
            enum __Field { __field0, __field1, __ignore, }
            impl ::serde::de::Deserialize for __Field {
                #[inline]
                fn deserialize<D>(deserializer: &mut D)
                 -> ::std::result::Result<__Field, D::Error> where
                 D: ::serde::de::Deserializer {
                    use std::marker::PhantomData;
                    struct __FieldVisitor<D> {
                        phantom: PhantomData<D>,
                    }
                    impl <__D> ::serde::de::Visitor for __FieldVisitor<__D>
                     where __D: ::serde::de::Deserializer {
                        type
                        Value
                        =
                        __Field;
                        fn visit_usize<E>(&mut self, value: usize)
                         -> ::std::result::Result<__Field, E> where
                         E: ::serde::de::Error {
                            match value {
                                0usize => { Ok(__Field::__field0) }
                                1usize => { Ok(__Field::__field1) }
                                _ => Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<E>(&mut self, value: &str)
                         -> ::std::result::Result<__Field, E> where
                         E: ::serde::de::Error {
                            match value {
                                "level" => { Ok(__Field::__field0) }
                                "appenders" => { Ok(__Field::__field1) }
                                _ => Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<E>(&mut self, value: &[u8])
                         -> ::std::result::Result<__Field, E> where
                         E: ::serde::de::Error {
                            match value {
                                b"level" => { Ok(__Field::__field0) }
                                b"appenders" => { Ok(__Field::__field1) }
                                _ => Ok(__Field::__ignore),
                            }
                        }
                    }
                    deserializer.deserialize_struct_field(__FieldVisitor::<D>{phantom:
                                                                                  PhantomData,})
                }
            }
            struct __Visitor<__D: ::serde::de::Deserializer>(::std::marker::PhantomData<__D>);
            impl <__D: ::serde::de::Deserializer> ::serde::de::Visitor for
             __Visitor<__D> {
                type
                Value
                =
                Root;
                #[inline]
                fn visit_seq<__V>(&mut self, mut visitor: __V)
                 -> ::std::result::Result<Root, __V::Error> where
                 __V: ::serde::de::SeqVisitor {
                    {
                        let __field0 =
                            match try!(visitor . visit (  )) {
                                Some(value) => { value }
                                None => {
                                    return Err(::serde::de::Error::end_of_stream());
                                }
                            };
                        let __field1 =
                            match try!(visitor . visit (  )) {
                                Some(value) => { value }
                                None => {
                                    return Err(::serde::de::Error::end_of_stream());
                                }
                            };
                        try!(visitor . end (  ));
                        Ok(Root{level: __field0, appenders: __field1,})
                    }
                }
                #[inline]
                fn visit_map<__V>(&mut self, mut visitor: __V)
                 -> ::std::result::Result<Root, __V::Error> where
                 __V: ::serde::de::MapVisitor {
                    {
                        let mut __field0 = None;
                        let mut __field1 = None;
                        while let Some(key) = try!(visitor . visit_key (  )) {
                            match key {
                                __Field::__field0 => {
                                    __field0 =
                                        Some(try!(visitor.visit_value()));
                                }
                                __Field::__field1 => {
                                    __field1 =
                                        Some(try!(visitor.visit_value()));
                                }
                                _ => {
                                    try!(visitor . visit_value:: < :: serde::
                                         de:: impls:: IgnoredAny > (  ));
                                }
                            }
                        }
                        let __field0 =
                            match __field0 {
                                Some(__field0) => __field0,
                                None =>
                                match visitor.missing_field("level") {
                                    ::std::result::Result::Ok(value) => value,
                                    ::std::result::Result::Err(value) =>
                                    return ::std::result::Result::Err(value),
                                },
                            };
                        let __field1 =
                            match __field1 {
                                Some(__field1) => __field1,
                                None => ::std::default::Default::default(),
                            };
                        try!(visitor . end (  ));
                        Ok(Root{level: __field0, appenders: __field1,})
                    }
                }
            }
            const FIELDS: &'static [&'static str] = &["level", "appenders"];
            deserializer.deserialize_struct("Root", FIELDS,
                                            __Visitor::<__D>(::std::marker::PhantomData))
        }
    }
}
#[derive_Debug]
#[derive_PartialEq]
pub struct Logger {
    pub level: DeLogLevelFilter,
    pub appenders: Vec<String>,
    pub additive: Option<bool>,
}
impl ::serde::de::Deserialize for Logger {
    fn deserialize<__D>(deserializer: &mut __D)
     -> ::std::result::Result<Logger, __D::Error> where
     __D: ::serde::de::Deserializer {
        {
            #[allow(non_camel_case_types)]
            enum __Field { __field0, __field1, __field2, __ignore, }
            impl ::serde::de::Deserialize for __Field {
                #[inline]
                fn deserialize<D>(deserializer: &mut D)
                 -> ::std::result::Result<__Field, D::Error> where
                 D: ::serde::de::Deserializer {
                    use std::marker::PhantomData;
                    struct __FieldVisitor<D> {
                        phantom: PhantomData<D>,
                    }
                    impl <__D> ::serde::de::Visitor for __FieldVisitor<__D>
                     where __D: ::serde::de::Deserializer {
                        type
                        Value
                        =
                        __Field;
                        fn visit_usize<E>(&mut self, value: usize)
                         -> ::std::result::Result<__Field, E> where
                         E: ::serde::de::Error {
                            match value {
                                0usize => { Ok(__Field::__field0) }
                                1usize => { Ok(__Field::__field1) }
                                2usize => { Ok(__Field::__field2) }
                                _ => Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<E>(&mut self, value: &str)
                         -> ::std::result::Result<__Field, E> where
                         E: ::serde::de::Error {
                            match value {
                                "level" => { Ok(__Field::__field0) }
                                "appenders" => { Ok(__Field::__field1) }
                                "additive" => { Ok(__Field::__field2) }
                                _ => Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<E>(&mut self, value: &[u8])
                         -> ::std::result::Result<__Field, E> where
                         E: ::serde::de::Error {
                            match value {
                                b"level" => { Ok(__Field::__field0) }
                                b"appenders" => { Ok(__Field::__field1) }
                                b"additive" => { Ok(__Field::__field2) }
                                _ => Ok(__Field::__ignore),
                            }
                        }
                    }
                    deserializer.deserialize_struct_field(__FieldVisitor::<D>{phantom:
                                                                                  PhantomData,})
                }
            }
            struct __Visitor<__D: ::serde::de::Deserializer>(::std::marker::PhantomData<__D>);
            impl <__D: ::serde::de::Deserializer> ::serde::de::Visitor for
             __Visitor<__D> {
                type
                Value
                =
                Logger;
                #[inline]
                fn visit_seq<__V>(&mut self, mut visitor: __V)
                 -> ::std::result::Result<Logger, __V::Error> where
                 __V: ::serde::de::SeqVisitor {
                    {
                        let __field0 =
                            match try!(visitor . visit (  )) {
                                Some(value) => { value }
                                None => {
                                    return Err(::serde::de::Error::end_of_stream());
                                }
                            };
                        let __field1 =
                            match try!(visitor . visit (  )) {
                                Some(value) => { value }
                                None => {
                                    return Err(::serde::de::Error::end_of_stream());
                                }
                            };
                        let __field2 =
                            match try!(visitor . visit (  )) {
                                Some(value) => { value }
                                None => {
                                    return Err(::serde::de::Error::end_of_stream());
                                }
                            };
                        try!(visitor . end (  ));
                        Ok(Logger{level: __field0,
                                  appenders: __field1,
                                  additive: __field2,})
                    }
                }
                #[inline]
                fn visit_map<__V>(&mut self, mut visitor: __V)
                 -> ::std::result::Result<Logger, __V::Error> where
                 __V: ::serde::de::MapVisitor {
                    {
                        let mut __field0 = None;
                        let mut __field1 = None;
                        let mut __field2 = None;
                        while let Some(key) = try!(visitor . visit_key (  )) {
                            match key {
                                __Field::__field0 => {
                                    __field0 =
                                        Some(try!(visitor.visit_value()));
                                }
                                __Field::__field1 => {
                                    __field1 =
                                        Some(try!(visitor.visit_value()));
                                }
                                __Field::__field2 => {
                                    __field2 =
                                        Some(try!(visitor.visit_value()));
                                }
                                _ => {
                                    try!(visitor . visit_value:: < :: serde::
                                         de:: impls:: IgnoredAny > (  ));
                                }
                            }
                        }
                        let __field0 =
                            match __field0 {
                                Some(__field0) => __field0,
                                None =>
                                match visitor.missing_field("level") {
                                    ::std::result::Result::Ok(value) => value,
                                    ::std::result::Result::Err(value) =>
                                    return ::std::result::Result::Err(value),
                                },
                            };
                        let __field1 =
                            match __field1 {
                                Some(__field1) => __field1,
                                None => ::std::default::Default::default(),
                            };
                        let __field2 =
                            match __field2 {
                                Some(__field2) => __field2,
                                None =>
                                match visitor.missing_field("additive") {
                                    ::std::result::Result::Ok(value) => value,
                                    ::std::result::Result::Err(value) =>
                                    return ::std::result::Result::Err(value),
                                },
                            };
                        try!(visitor . end (  ));
                        Ok(Logger{level: __field0,
                                  appenders: __field1,
                                  additive: __field2,})
                    }
                }
            }
            const FIELDS: &'static [&'static str] =
                &["level", "appenders", "additive"];
            deserializer.deserialize_struct("Logger", FIELDS,
                                            __Visitor::<__D>(::std::marker::PhantomData))
        }
    }
}
pub struct FileAppenderConfig {
    pub path: String,
    pub encoder: Option<Encoder>,
    pub append: Option<bool>,
}
impl ::serde::de::Deserialize for FileAppenderConfig {
    fn deserialize<__D>(deserializer: &mut __D)
     -> ::std::result::Result<FileAppenderConfig, __D::Error> where
     __D: ::serde::de::Deserializer {
        {
            #[allow(non_camel_case_types)]
            enum __Field { __field0, __field1, __field2, __ignore, }
            impl ::serde::de::Deserialize for __Field {
                #[inline]
                fn deserialize<D>(deserializer: &mut D)
                 -> ::std::result::Result<__Field, D::Error> where
                 D: ::serde::de::Deserializer {
                    use std::marker::PhantomData;
                    struct __FieldVisitor<D> {
                        phantom: PhantomData<D>,
                    }
                    impl <__D> ::serde::de::Visitor for __FieldVisitor<__D>
                     where __D: ::serde::de::Deserializer {
                        type
                        Value
                        =
                        __Field;
                        fn visit_usize<E>(&mut self, value: usize)
                         -> ::std::result::Result<__Field, E> where
                         E: ::serde::de::Error {
                            match value {
                                0usize => { Ok(__Field::__field0) }
                                1usize => { Ok(__Field::__field1) }
                                2usize => { Ok(__Field::__field2) }
                                _ => Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<E>(&mut self, value: &str)
                         -> ::std::result::Result<__Field, E> where
                         E: ::serde::de::Error {
                            match value {
                                "path" => { Ok(__Field::__field0) }
                                "encoder" => { Ok(__Field::__field1) }
                                "append" => { Ok(__Field::__field2) }
                                _ => Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<E>(&mut self, value: &[u8])
                         -> ::std::result::Result<__Field, E> where
                         E: ::serde::de::Error {
                            match value {
                                b"path" => { Ok(__Field::__field0) }
                                b"encoder" => { Ok(__Field::__field1) }
                                b"append" => { Ok(__Field::__field2) }
                                _ => Ok(__Field::__ignore),
                            }
                        }
                    }
                    deserializer.deserialize_struct_field(__FieldVisitor::<D>{phantom:
                                                                                  PhantomData,})
                }
            }
            struct __Visitor<__D: ::serde::de::Deserializer>(::std::marker::PhantomData<__D>);
            impl <__D: ::serde::de::Deserializer> ::serde::de::Visitor for
             __Visitor<__D> {
                type
                Value
                =
                FileAppenderConfig;
                #[inline]
                fn visit_seq<__V>(&mut self, mut visitor: __V)
                 -> ::std::result::Result<FileAppenderConfig, __V::Error>
                 where __V: ::serde::de::SeqVisitor {
                    {
                        let __field0 =
                            match try!(visitor . visit (  )) {
                                Some(value) => { value }
                                None => {
                                    return Err(::serde::de::Error::end_of_stream());
                                }
                            };
                        let __field1 =
                            match try!(visitor . visit (  )) {
                                Some(value) => { value }
                                None => {
                                    return Err(::serde::de::Error::end_of_stream());
                                }
                            };
                        let __field2 =
                            match try!(visitor . visit (  )) {
                                Some(value) => { value }
                                None => {
                                    return Err(::serde::de::Error::end_of_stream());
                                }
                            };
                        try!(visitor . end (  ));
                        Ok(FileAppenderConfig{path: __field0,
                                              encoder: __field1,
                                              append: __field2,})
                    }
                }
                #[inline]
                fn visit_map<__V>(&mut self, mut visitor: __V)
                 -> ::std::result::Result<FileAppenderConfig, __V::Error>
                 where __V: ::serde::de::MapVisitor {
                    {
                        let mut __field0 = None;
                        let mut __field1 = None;
                        let mut __field2 = None;
                        while let Some(key) = try!(visitor . visit_key (  )) {
                            match key {
                                __Field::__field0 => {
                                    __field0 =
                                        Some(try!(visitor.visit_value()));
                                }
                                __Field::__field1 => {
                                    __field1 =
                                        Some(try!(visitor.visit_value()));
                                }
                                __Field::__field2 => {
                                    __field2 =
                                        Some(try!(visitor.visit_value()));
                                }
                                _ => {
                                    try!(visitor . visit_value:: < :: serde::
                                         de:: impls:: IgnoredAny > (  ));
                                }
                            }
                        }
                        let __field0 =
                            match __field0 {
                                Some(__field0) => __field0,
                                None =>
                                match visitor.missing_field("path") {
                                    ::std::result::Result::Ok(value) => value,
                                    ::std::result::Result::Err(value) =>
                                    return ::std::result::Result::Err(value),
                                },
                            };
                        let __field1 =
                            match __field1 {
                                Some(__field1) => __field1,
                                None => ::std::default::Default::default(),
                            };
                        let __field2 =
                            match __field2 {
                                Some(__field2) => __field2,
                                None => ::std::default::Default::default(),
                            };
                        try!(visitor . end (  ));
                        Ok(FileAppenderConfig{path: __field0,
                                              encoder: __field1,
                                              append: __field2,})
                    }
                }
            }
            const FIELDS: &'static [&'static str] =
                &["path", "encoder", "append"];
            deserializer.deserialize_struct("FileAppenderConfig", FIELDS,
                                            __Visitor::<__D>(::std::marker::PhantomData))
        }
    }
}
pub struct ConsoleAppenderConfig {
    pub encoder: Option<Encoder>,
}
impl ::serde::de::Deserialize for ConsoleAppenderConfig {
    fn deserialize<__D>(deserializer: &mut __D)
     -> ::std::result::Result<ConsoleAppenderConfig, __D::Error> where
     __D: ::serde::de::Deserializer {
        {
            #[allow(non_camel_case_types)]
            enum __Field { __field0, __ignore, }
            impl ::serde::de::Deserialize for __Field {
                #[inline]
                fn deserialize<D>(deserializer: &mut D)
                 -> ::std::result::Result<__Field, D::Error> where
                 D: ::serde::de::Deserializer {
                    use std::marker::PhantomData;
                    struct __FieldVisitor<D> {
                        phantom: PhantomData<D>,
                    }
                    impl <__D> ::serde::de::Visitor for __FieldVisitor<__D>
                     where __D: ::serde::de::Deserializer {
                        type
                        Value
                        =
                        __Field;
                        fn visit_usize<E>(&mut self, value: usize)
                         -> ::std::result::Result<__Field, E> where
                         E: ::serde::de::Error {
                            match value {
                                0usize => { Ok(__Field::__field0) }
                                _ => Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<E>(&mut self, value: &str)
                         -> ::std::result::Result<__Field, E> where
                         E: ::serde::de::Error {
                            match value {
                                "encoder" => { Ok(__Field::__field0) }
                                _ => Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<E>(&mut self, value: &[u8])
                         -> ::std::result::Result<__Field, E> where
                         E: ::serde::de::Error {
                            match value {
                                b"encoder" => { Ok(__Field::__field0) }
                                _ => Ok(__Field::__ignore),
                            }
                        }
                    }
                    deserializer.deserialize_struct_field(__FieldVisitor::<D>{phantom:
                                                                                  PhantomData,})
                }
            }
            struct __Visitor<__D: ::serde::de::Deserializer>(::std::marker::PhantomData<__D>);
            impl <__D: ::serde::de::Deserializer> ::serde::de::Visitor for
             __Visitor<__D> {
                type
                Value
                =
                ConsoleAppenderConfig;
                #[inline]
                fn visit_seq<__V>(&mut self, mut visitor: __V)
                 -> ::std::result::Result<ConsoleAppenderConfig, __V::Error>
                 where __V: ::serde::de::SeqVisitor {
                    {
                        let __field0 =
                            match try!(visitor . visit (  )) {
                                Some(value) => { value }
                                None => {
                                    return Err(::serde::de::Error::end_of_stream());
                                }
                            };
                        try!(visitor . end (  ));
                        Ok(ConsoleAppenderConfig{encoder: __field0,})
                    }
                }
                #[inline]
                fn visit_map<__V>(&mut self, mut visitor: __V)
                 -> ::std::result::Result<ConsoleAppenderConfig, __V::Error>
                 where __V: ::serde::de::MapVisitor {
                    {
                        let mut __field0 = None;
                        while let Some(key) = try!(visitor . visit_key (  )) {
                            match key {
                                __Field::__field0 => {
                                    __field0 =
                                        Some(try!(visitor.visit_value()));
                                }
                                _ => {
                                    try!(visitor . visit_value:: < :: serde::
                                         de:: impls:: IgnoredAny > (  ));
                                }
                            }
                        }
                        let __field0 =
                            match __field0 {
                                Some(__field0) => __field0,
                                None => ::std::default::Default::default(),
                            };
                        try!(visitor . end (  ));
                        Ok(ConsoleAppenderConfig{encoder: __field0,})
                    }
                }
            }
            const FIELDS: &'static [&'static str] = &["encoder"];
            deserializer.deserialize_struct("ConsoleAppenderConfig", FIELDS,
                                            __Visitor::<__D>(::std::marker::PhantomData))
        }
    }
}
pub struct ThresholdFilterConfig {
    pub level: DeLogLevelFilter,
}
impl ::serde::de::Deserialize for ThresholdFilterConfig {
    fn deserialize<__D>(deserializer: &mut __D)
     -> ::std::result::Result<ThresholdFilterConfig, __D::Error> where
     __D: ::serde::de::Deserializer {
        {
            #[allow(non_camel_case_types)]
            enum __Field { __field0, __ignore, }
            impl ::serde::de::Deserialize for __Field {
                #[inline]
                fn deserialize<D>(deserializer: &mut D)
                 -> ::std::result::Result<__Field, D::Error> where
                 D: ::serde::de::Deserializer {
                    use std::marker::PhantomData;
                    struct __FieldVisitor<D> {
                        phantom: PhantomData<D>,
                    }
                    impl <__D> ::serde::de::Visitor for __FieldVisitor<__D>
                     where __D: ::serde::de::Deserializer {
                        type
                        Value
                        =
                        __Field;
                        fn visit_usize<E>(&mut self, value: usize)
                         -> ::std::result::Result<__Field, E> where
                         E: ::serde::de::Error {
                            match value {
                                0usize => { Ok(__Field::__field0) }
                                _ => Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<E>(&mut self, value: &str)
                         -> ::std::result::Result<__Field, E> where
                         E: ::serde::de::Error {
                            match value {
                                "level" => { Ok(__Field::__field0) }
                                _ => Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<E>(&mut self, value: &[u8])
                         -> ::std::result::Result<__Field, E> where
                         E: ::serde::de::Error {
                            match value {
                                b"level" => { Ok(__Field::__field0) }
                                _ => Ok(__Field::__ignore),
                            }
                        }
                    }
                    deserializer.deserialize_struct_field(__FieldVisitor::<D>{phantom:
                                                                                  PhantomData,})
                }
            }
            struct __Visitor<__D: ::serde::de::Deserializer>(::std::marker::PhantomData<__D>);
            impl <__D: ::serde::de::Deserializer> ::serde::de::Visitor for
             __Visitor<__D> {
                type
                Value
                =
                ThresholdFilterConfig;
                #[inline]
                fn visit_seq<__V>(&mut self, mut visitor: __V)
                 -> ::std::result::Result<ThresholdFilterConfig, __V::Error>
                 where __V: ::serde::de::SeqVisitor {
                    {
                        let __field0 =
                            match try!(visitor . visit (  )) {
                                Some(value) => { value }
                                None => {
                                    return Err(::serde::de::Error::end_of_stream());
                                }
                            };
                        try!(visitor . end (  ));
                        Ok(ThresholdFilterConfig{level: __field0,})
                    }
                }
                #[inline]
                fn visit_map<__V>(&mut self, mut visitor: __V)
                 -> ::std::result::Result<ThresholdFilterConfig, __V::Error>
                 where __V: ::serde::de::MapVisitor {
                    {
                        let mut __field0 = None;
                        while let Some(key) = try!(visitor . visit_key (  )) {
                            match key {
                                __Field::__field0 => {
                                    __field0 =
                                        Some(try!(visitor.visit_value()));
                                }
                                _ => {
                                    try!(visitor . visit_value:: < :: serde::
                                         de:: impls:: IgnoredAny > (  ));
                                }
                            }
                        }
                        let __field0 =
                            match __field0 {
                                Some(__field0) => __field0,
                                None =>
                                match visitor.missing_field("level") {
                                    ::std::result::Result::Ok(value) => value,
                                    ::std::result::Result::Err(value) =>
                                    return ::std::result::Result::Err(value),
                                },
                            };
                        try!(visitor . end (  ));
                        Ok(ThresholdFilterConfig{level: __field0,})
                    }
                }
            }
            const FIELDS: &'static [&'static str] = &["level"];
            deserializer.deserialize_struct("ThresholdFilterConfig", FIELDS,
                                            __Visitor::<__D>(::std::marker::PhantomData))
        }
    }
}
pub struct PatternEncoderConfig {
    pub pattern: Option<String>,
}
impl ::serde::de::Deserialize for PatternEncoderConfig {
    fn deserialize<__D>(deserializer: &mut __D)
     -> ::std::result::Result<PatternEncoderConfig, __D::Error> where
     __D: ::serde::de::Deserializer {
        {
            #[allow(non_camel_case_types)]
            enum __Field { __field0, __ignore, }
            impl ::serde::de::Deserialize for __Field {
                #[inline]
                fn deserialize<D>(deserializer: &mut D)
                 -> ::std::result::Result<__Field, D::Error> where
                 D: ::serde::de::Deserializer {
                    use std::marker::PhantomData;
                    struct __FieldVisitor<D> {
                        phantom: PhantomData<D>,
                    }
                    impl <__D> ::serde::de::Visitor for __FieldVisitor<__D>
                     where __D: ::serde::de::Deserializer {
                        type
                        Value
                        =
                        __Field;
                        fn visit_usize<E>(&mut self, value: usize)
                         -> ::std::result::Result<__Field, E> where
                         E: ::serde::de::Error {
                            match value {
                                0usize => { Ok(__Field::__field0) }
                                _ => Ok(__Field::__ignore),
                            }
                        }
                        fn visit_str<E>(&mut self, value: &str)
                         -> ::std::result::Result<__Field, E> where
                         E: ::serde::de::Error {
                            match value {
                                "pattern" => { Ok(__Field::__field0) }
                                _ => Ok(__Field::__ignore),
                            }
                        }
                        fn visit_bytes<E>(&mut self, value: &[u8])
                         -> ::std::result::Result<__Field, E> where
                         E: ::serde::de::Error {
                            match value {
                                b"pattern" => { Ok(__Field::__field0) }
                                _ => Ok(__Field::__ignore),
                            }
                        }
                    }
                    deserializer.deserialize_struct_field(__FieldVisitor::<D>{phantom:
                                                                                  PhantomData,})
                }
            }
            struct __Visitor<__D: ::serde::de::Deserializer>(::std::marker::PhantomData<__D>);
            impl <__D: ::serde::de::Deserializer> ::serde::de::Visitor for
             __Visitor<__D> {
                type
                Value
                =
                PatternEncoderConfig;
                #[inline]
                fn visit_seq<__V>(&mut self, mut visitor: __V)
                 -> ::std::result::Result<PatternEncoderConfig, __V::Error>
                 where __V: ::serde::de::SeqVisitor {
                    {
                        let __field0 =
                            match try!(visitor . visit (  )) {
                                Some(value) => { value }
                                None => {
                                    return Err(::serde::de::Error::end_of_stream());
                                }
                            };
                        try!(visitor . end (  ));
                        Ok(PatternEncoderConfig{pattern: __field0,})
                    }
                }
                #[inline]
                fn visit_map<__V>(&mut self, mut visitor: __V)
                 -> ::std::result::Result<PatternEncoderConfig, __V::Error>
                 where __V: ::serde::de::MapVisitor {
                    {
                        let mut __field0 = None;
                        while let Some(key) = try!(visitor . visit_key (  )) {
                            match key {
                                __Field::__field0 => {
                                    __field0 =
                                        Some(try!(visitor.visit_value()));
                                }
                                _ => {
                                    try!(visitor . visit_value:: < :: serde::
                                         de:: impls:: IgnoredAny > (  ));
                                }
                            }
                        }
                        let __field0 =
                            match __field0 {
                                Some(__field0) => __field0,
                                None => ::std::default::Default::default(),
                            };
                        try!(visitor . end (  ));
                        Ok(PatternEncoderConfig{pattern: __field0,})
                    }
                }
            }
            const FIELDS: &'static [&'static str] = &["pattern"];
            deserializer.deserialize_struct("PatternEncoderConfig", FIELDS,
                                            __Visitor::<__D>(::std::marker::PhantomData))
        }
    }
}
