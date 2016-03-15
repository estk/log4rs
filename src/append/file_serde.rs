struct FileAppenderConfig {
    path: String,
    encoder: Option<Encoder>,
    append: Option<bool>,
}
impl ::serde::de::Deserialize for FileAppenderConfig {
    fn deserialize<__D>(deserializer: &mut __D)
     -> ::std::result::Result<FileAppenderConfig, __D::Error> where
     __D: ::serde::de::Deserializer {
        {
            #[allow(non_camel_case_types)]
            enum __Field { __field0, __field1, __field2, }
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
                                _ => {
                                    Err(::serde::de::Error::invalid_value("expected a field"))
                                }
                            }
                        }
                        fn visit_str<E>(&mut self, value: &str)
                         -> ::std::result::Result<__Field, E> where
                         E: ::serde::de::Error {
                            match value {
                                "path" => { Ok(__Field::__field0) }
                                "encoder" => { Ok(__Field::__field1) }
                                "append" => { Ok(__Field::__field2) }
                                _ =>
                                Err(::serde::de::Error::unknown_field(value)),
                            }
                        }
                        fn visit_bytes<E>(&mut self, value: &[u8])
                         -> ::std::result::Result<__Field, E> where
                         E: ::serde::de::Error {
                            match value {
                                b"path" => { Ok(__Field::__field0) }
                                b"encoder" => { Ok(__Field::__field1) }
                                b"append" => { Ok(__Field::__field2) }
                                _ => {
                                    let value =
                                        ::std::string::String::from_utf8_lossy(value);
                                    Err(::serde::de::Error::unknown_field(&value))
                                }
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
                                None =>
                                match visitor.missing_field("encoder") {
                                    ::std::result::Result::Ok(value) => value,
                                    ::std::result::Result::Err(value) =>
                                    return ::std::result::Result::Err(value),
                                },
                            };
                        let __field2 =
                            match __field2 {
                                Some(__field2) => __field2,
                                None =>
                                match visitor.missing_field("append") {
                                    ::std::result::Result::Ok(value) => value,
                                    ::std::result::Result::Err(value) =>
                                    return ::std::result::Result::Err(value),
                                },
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
