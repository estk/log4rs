use log::LogLevelFilter;

#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DESERIALIZE_FOR_ThresholdFilterConfig: () =
    {
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::de::Deserialize for ThresholdFilterConfig {
            fn deserialize<__D>(deserializer: &mut __D)
             -> ::std::result::Result<ThresholdFilterConfig, __D::Error> where
             __D: _serde::de::Deserializer {
                #[allow(non_camel_case_types)]
                enum __Field { __field0, __ignore, }
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
                        deserializer.deserialize_struct_field(__FieldVisitor)
                    }
                }
                struct __Visitor;
                impl _serde::de::Visitor for __Visitor {
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
                        let __field0 =
                            match {
                                      struct __SerdeDeserializeWithStruct {
                                          value: LogLevelFilter,
                                          phantom: ::std::marker::PhantomData<ThresholdFilterConfig>,
                                      }
                                      impl _serde::de::Deserialize for
                                       __SerdeDeserializeWithStruct {
                                          fn deserialize<__D>(__d: &mut __D)
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
                                           __SerdeDeserializeWithStruct > (
                                           )).map(|wrap| wrap.value)
                                  } {
                                Some(value) => { value }
                                None => {
                                    try!(visitor . end (  ));
                                    return Err(_serde::de::Error::invalid_length(0usize));
                                }
                            };
                        try!(visitor . end (  ));
                        Ok(ThresholdFilterConfig{level: __field0,})
                    }
                    #[inline]
                    fn visit_map<__V>(&mut self, mut visitor: __V)
                     ->
                         ::std::result::Result<ThresholdFilterConfig,
                                               __V::Error> where
                     __V: _serde::de::MapVisitor {
                        let mut __field0: Option<LogLevelFilter> = None;
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
                                        Some(({
                                                  struct __SerdeDeserializeWithStruct {
                                                      value: LogLevelFilter,
                                                      phantom: ::std::marker::PhantomData<ThresholdFilterConfig>,
                                                  }
                                                  impl _serde::de::Deserialize
                                                   for
                                                   __SerdeDeserializeWithStruct
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
                                                                   ( __d ));
                                                          Ok(__SerdeDeserializeWithStruct{value:
                                                                                              value,
                                                                                          phantom:
                                                                                              ::std::marker::PhantomData,})
                                                      }
                                                  }
                                                  try!(visitor . visit_value
                                                       :: <
                                                       __SerdeDeserializeWithStruct
                                                       > (  )).value
                                              }));
                                }
                                _ => {
                                    try!(visitor . visit_value :: < _serde ::
                                         de :: impls :: IgnoredAny > (  ));
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
                        Ok(ThresholdFilterConfig{level: __field0,})
                    }
                }
                const FIELDS: &'static [&'static str] = &["level"];
                deserializer.deserialize_struct("ThresholdFilterConfig",
                                                FIELDS, __Visitor)
            }
        }
    };
pub struct ThresholdFilterConfig {
    pub level: LogLevelFilter,
}
