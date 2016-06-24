pub struct FormatConfig {
    pub facility: Option<String>,
    pub hostname: Option<String>,
    pub app_name: Option<String>,
    pub procid: Option<String>,
    pub bom: Option<bool>,
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DESERIALIZE_FOR_FormatConfig: () =
    {
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::de::Deserialize for FormatConfig {
            fn deserialize<__D>(deserializer: &mut __D)
             -> ::std::result::Result<FormatConfig, __D::Error> where
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
                                        "facility" => {
                                            Ok(__Field::__field0)
                                        }
                                        "hostname" => {
                                            Ok(__Field::__field1)
                                        }
                                        "app_name" => {
                                            Ok(__Field::__field2)
                                        }
                                        "procid" => { Ok(__Field::__field3) }
                                        "bom" => { Ok(__Field::__field4) }
                                        _ =>
                                        Err(_serde::de::Error::unknown_field(value)),
                                    }
                                }
                                fn visit_bytes<__E>(&mut self, value: &[u8])
                                 -> ::std::result::Result<__Field, __E> where
                                 __E: _serde::de::Error {
                                    match value {
                                        b"facility" => {
                                            Ok(__Field::__field0)
                                        }
                                        b"hostname" => {
                                            Ok(__Field::__field1)
                                        }
                                        b"app_name" => {
                                            Ok(__Field::__field2)
                                        }
                                        b"procid" => { Ok(__Field::__field3) }
                                        b"bom" => { Ok(__Field::__field4) }
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
                    struct __Visitor<__D: _serde::de::Deserializer>(::std::marker::PhantomData<__D>);
                    impl <__D: _serde::de::Deserializer> _serde::de::Visitor
                     for __Visitor<__D> {
                        type
                        Value
                        =
                        FormatConfig;
                        #[inline]
                        fn visit_seq<__V>(&mut self, mut visitor: __V)
                         -> ::std::result::Result<FormatConfig, __V::Error>
                         where __V: _serde::de::SeqVisitor {
                            {
                                let __field0 =
                                    match try!(visitor . visit :: <
                                               Option<String> > (  )) {
                                        Some(value) => { value }
                                        None => {
                                            return Err(_serde::de::Error::end_of_stream());
                                        }
                                    };
                                let __field1 =
                                    match try!(visitor . visit :: <
                                               Option<String> > (  )) {
                                        Some(value) => { value }
                                        None => {
                                            return Err(_serde::de::Error::end_of_stream());
                                        }
                                    };
                                let __field2 =
                                    match try!(visitor . visit :: <
                                               Option<String> > (  )) {
                                        Some(value) => { value }
                                        None => {
                                            return Err(_serde::de::Error::end_of_stream());
                                        }
                                    };
                                let __field3 =
                                    match try!(visitor . visit :: <
                                               Option<String> > (  )) {
                                        Some(value) => { value }
                                        None => {
                                            return Err(_serde::de::Error::end_of_stream());
                                        }
                                    };
                                let __field4 =
                                    match try!(visitor . visit :: <
                                               Option<bool> > (  )) {
                                        Some(value) => { value }
                                        None => {
                                            return Err(_serde::de::Error::end_of_stream());
                                        }
                                    };
                                try!(visitor . end (  ));
                                Ok(FormatConfig{facility: __field0,
                                                hostname: __field1,
                                                app_name: __field2,
                                                procid: __field3,
                                                bom: __field4,})
                            }
                        }
                        #[inline]
                        fn visit_map<__V>(&mut self, mut visitor: __V)
                         -> ::std::result::Result<FormatConfig, __V::Error>
                         where __V: _serde::de::MapVisitor {
                            {
                                let mut __field0: Option<Option<String>> =
                                    None;
                                let mut __field1: Option<Option<String>> =
                                    None;
                                let mut __field2: Option<Option<String>> =
                                    None;
                                let mut __field3: Option<Option<String>> =
                                    None;
                                let mut __field4: Option<Option<bool>> = None;
                                while let Some(key) =
                                          try!(visitor . visit_key :: <
                                               __Field > (  )) {
                                    match key {
                                        __Field::__field0 => {
                                            if __field0.is_some() {
                                                return Err(<__V::Error as
                                                               _serde::de::Error>::duplicate_field("facility"));
                                            }
                                            __field0 =
                                                Some(try!(visitor .
                                                          visit_value :: <
                                                          Option<String> > (
                                                          )));
                                        }
                                        __Field::__field1 => {
                                            if __field1.is_some() {
                                                return Err(<__V::Error as
                                                               _serde::de::Error>::duplicate_field("hostname"));
                                            }
                                            __field1 =
                                                Some(try!(visitor .
                                                          visit_value :: <
                                                          Option<String> > (
                                                          )));
                                        }
                                        __Field::__field2 => {
                                            if __field2.is_some() {
                                                return Err(<__V::Error as
                                                               _serde::de::Error>::duplicate_field("app_name"));
                                            }
                                            __field2 =
                                                Some(try!(visitor .
                                                          visit_value :: <
                                                          Option<String> > (
                                                          )));
                                        }
                                        __Field::__field3 => {
                                            if __field3.is_some() {
                                                return Err(<__V::Error as
                                                               _serde::de::Error>::duplicate_field("procid"));
                                            }
                                            __field3 =
                                                Some(try!(visitor .
                                                          visit_value :: <
                                                          Option<String> > (
                                                          )));
                                        }
                                        __Field::__field4 => {
                                            if __field4.is_some() {
                                                return Err(<__V::Error as
                                                               _serde::de::Error>::duplicate_field("bom"));
                                            }
                                            __field4 =
                                                Some(try!(visitor .
                                                          visit_value :: <
                                                          Option<bool> > (
                                                          )));
                                        }
                                    }
                                }
                                let __field0 =
                                    match __field0 {
                                        Some(__field0) => __field0,
                                        None =>
                                        try!(visitor . missing_field (
                                             "facility" )),
                                    };
                                let __field1 =
                                    match __field1 {
                                        Some(__field1) => __field1,
                                        None =>
                                        try!(visitor . missing_field (
                                             "hostname" )),
                                    };
                                let __field2 =
                                    match __field2 {
                                        Some(__field2) => __field2,
                                        None =>
                                        try!(visitor . missing_field (
                                             "app_name" )),
                                    };
                                let __field3 =
                                    match __field3 {
                                        Some(__field3) => __field3,
                                        None =>
                                        try!(visitor . missing_field (
                                             "procid" )),
                                    };
                                let __field4 =
                                    match __field4 {
                                        Some(__field4) => __field4,
                                        None =>
                                        try!(visitor . missing_field ( "bom"
                                             )),
                                    };
                                try!(visitor . end (  ));
                                Ok(FormatConfig{facility: __field0,
                                                hostname: __field1,
                                                app_name: __field2,
                                                procid: __field3,
                                                bom: __field4,})
                            }
                        }
                    }
                    const FIELDS: &'static [&'static str] =
                        &["facility", "hostname", "app_name", "procid",
                          "bom"];
                    deserializer.deserialize_struct("FormatConfig", FIELDS,
                                                    __Visitor::<__D>(::std::marker::PhantomData))
                }
            }
        }
    };
