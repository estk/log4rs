#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DESERIALIZE_FOR_ThresholdFilterConfig: () =
    {
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Deserialize for ThresholdFilterConfig {
            fn deserialize<__D>(deserializer: __D)
             -> _serde::export::Result<ThresholdFilterConfig, __D::Error>
             where __D: _serde::Deserializer {
                #[allow(non_camel_case_types)]
                enum __Field { __field0, __ignore, }
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
                                    _ => Ok(__Field::__ignore),
                                }
                            }
                            fn visit_bytes<__E>(self, value: &[u8])
                             -> _serde::export::Result<__Field, __E> where
                             __E: _serde::de::Error {
                                match value {
                                    b"level" => Ok(__Field::__field0),
                                    _ => { Ok(__Field::__ignore) }
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
                    fn expecting(&self,
                                 formatter:
                                     &mut _serde::export::fmt::Formatter)
                     -> _serde::export::fmt::Result {
                        formatter.write_str("struct ThresholdFilterConfig")
                    }
                    #[inline]
                    fn visit_seq<__V>(self, mut visitor: __V)
                     ->
                         _serde::export::Result<ThresholdFilterConfig,
                                                __V::Error> where
                     __V: _serde::de::SeqVisitor {
                        let __field0 =
                            match {
                                      struct __SerdeDeserializeWithStruct {
                                          value: LogLevelFilter,
                                          phantom: _serde::export::PhantomData<ThresholdFilterConfig>,
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
                                                                                 &"tuple of 1 elements"));
                                }
                            };
                        Ok(ThresholdFilterConfig{level: __field0,})
                    }
                    #[inline]
                    fn visit_map<__V>(self, mut visitor: __V)
                     ->
                         _serde::export::Result<ThresholdFilterConfig,
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
                                        Some({
                                                 struct __SerdeDeserializeWithStruct {
                                                     value: LogLevelFilter,
                                                     phantom: _serde::export::PhantomData<ThresholdFilterConfig>,
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
                                _ => {
                                    let _ =
                                        try!(visitor . visit_value :: < _serde
                                             :: de :: impls :: IgnoredAny > (
                                             ));
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
                        Ok(ThresholdFilterConfig{level: __field0,})
                    }
                }
                const FIELDS: &'static [&'static str] = &["level"];
                deserializer.deserialize_struct("ThresholdFilterConfig",
                                                FIELDS, __Visitor)
            }
        }
    };
/// The threshold filter's configuration.
pub struct ThresholdFilterConfig {
    level: LogLevelFilter,
}
