use priv_serde::DeLogLevelFilter;

pub struct ThresholdFilterConfig {
    pub level: DeLogLevelFilter,
}
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DESERIALIZE_FOR_ThresholdFilterConfig: () =
    {
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::de::Deserialize for ThresholdFilterConfig {
            fn deserialize<__D>(deserializer: &mut __D)
             -> ::std::result::Result<ThresholdFilterConfig, __D::Error> where
             __D: _serde::de::Deserializer {
                {
                    #[allow(non_camel_case_types)]
                    enum __Field { __field0, __ignore, }
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
                                        _ => Ok(__Field::__ignore),
                                    }
                                }
                                fn visit_str<__E>(&mut self, value: &str)
                                 -> ::std::result::Result<__Field, __E> where
                                 __E: _serde::de::Error {
                                    match value {
                                        "level" => { Ok(__Field::__field0) }
                                        _ => Ok(__Field::__ignore),
                                    }
                                }
                                fn visit_bytes<__E>(&mut self, value: &[u8])
                                 -> ::std::result::Result<__Field, __E> where
                                 __E: _serde::de::Error {
                                    match value {
                                        b"level" => { Ok(__Field::__field0) }
                                        _ => Ok(__Field::__ignore),
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
                        ThresholdFilterConfig;
                        #[inline]
                        fn visit_seq<__V>(&mut self, mut visitor: __V)
                         ->
                             ::std::result::Result<ThresholdFilterConfig,
                                                   __V::Error> where
                         __V: _serde::de::SeqVisitor {
                            {
                                let __field0 =
                                    match try!(visitor . visit :: <
                                               DeLogLevelFilter > (  )) {
                                        Some(value) => { value }
                                        None => {
                                            return Err(_serde::de::Error::end_of_stream());
                                        }
                                    };
                                try!(visitor . end (  ));
                                Ok(ThresholdFilterConfig{level: __field0,})
                            }
                        }
                        #[inline]
                        fn visit_map<__V>(&mut self, mut visitor: __V)
                         ->
                             ::std::result::Result<ThresholdFilterConfig,
                                                   __V::Error> where
                         __V: _serde::de::MapVisitor {
                            {
                                let mut __field0: Option<DeLogLevelFilter> =
                                    None;
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
                                                Some(try!(visitor .
                                                          visit_value :: <
                                                          DeLogLevelFilter > (
                                                           )));
                                        }
                                        _ => {
                                            try!(visitor . visit_value :: <
                                                 _serde :: de :: impls ::
                                                 IgnoredAny > (  ));
                                        }
                                    }
                                }
                                let __field0 =
                                    match __field0 {
                                        Some(__field0) => __field0,
                                        None =>
                                        try!(visitor . missing_field ( "level"
                                             )),
                                    };
                                try!(visitor . end (  ));
                                Ok(ThresholdFilterConfig{level: __field0,})
                            }
                        }
                    }
                    const FIELDS: &'static [&'static str] = &["level"];
                    deserializer.deserialize_struct("ThresholdFilterConfig",
                                                    FIELDS,
                                                    __Visitor::<__D>(::std::marker::PhantomData))
                }
            }
        }
    };
