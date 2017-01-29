#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DESERIALIZE_FOR_SizeTriggerConfig: () =
    {
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Deserialize for SizeTriggerConfig {
            fn deserialize<__D>(deserializer: __D)
             -> _serde::export::Result<SizeTriggerConfig, __D::Error> where
             __D: _serde::Deserializer {
                #[allow(non_camel_case_types)]
                enum __Field { __field0, }
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
                                    "limit" => Ok(__Field::__field0),
                                    _ =>
                                    Err(_serde::de::Error::unknown_field(value,
                                                                         FIELDS)),
                                }
                            }
                            fn visit_bytes<__E>(self, value: &[u8])
                             -> _serde::export::Result<__Field, __E> where
                             __E: _serde::de::Error {
                                match value {
                                    b"limit" => Ok(__Field::__field0),
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
                    SizeTriggerConfig;
                    fn expecting(&self,
                                 formatter:
                                     &mut _serde::export::fmt::Formatter)
                     -> _serde::export::fmt::Result {
                        formatter.write_str("struct SizeTriggerConfig")
                    }
                    #[inline]
                    fn visit_seq<__V>(self, mut visitor: __V)
                     -> _serde::export::Result<SizeTriggerConfig, __V::Error>
                     where __V: _serde::de::SeqVisitor {
                        let __field0 =
                            match {
                                      struct __SerdeDeserializeWithStruct {
                                          value: u64,
                                          phantom: _serde::export::PhantomData<SizeTriggerConfig>,
                                      }
                                      impl _serde::Deserialize for
                                       __SerdeDeserializeWithStruct {
                                          fn deserialize<__D>(__d: __D)
                                           ->
                                               _serde::export::Result<Self,
                                                                      __D::Error>
                                           where __D: _serde::Deserializer {
                                              let value =
                                                  try!(deserialize_limit ( __d
                                                       ));
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
                        Ok(SizeTriggerConfig{limit: __field0,})
                    }
                    #[inline]
                    fn visit_map<__V>(self, mut visitor: __V)
                     -> _serde::export::Result<SizeTriggerConfig, __V::Error>
                     where __V: _serde::de::MapVisitor {
                        let mut __field0: Option<u64> = None;
                        while let Some(key) =
                                  try!(visitor . visit_key :: < __Field > (
                                       )) {
                            match key {
                                __Field::__field0 => {
                                    if __field0.is_some() {
                                        return Err(<__V::Error as
                                                       _serde::de::Error>::duplicate_field("limit"));
                                    }
                                    __field0 =
                                        Some({
                                                 struct __SerdeDeserializeWithStruct {
                                                     value: u64,
                                                     phantom: _serde::export::PhantomData<SizeTriggerConfig>,
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
                                                             try!(deserialize_limit
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
                            }
                        }
                        let __field0 =
                            match __field0 {
                                Some(__field0) => __field0,
                                None =>
                                return Err(<__V::Error as
                                               _serde::de::Error>::missing_field("limit")),
                            };
                        Ok(SizeTriggerConfig{limit: __field0,})
                    }
                }
                const FIELDS: &'static [&'static str] = &["limit"];
                deserializer.deserialize_struct("SizeTriggerConfig", FIELDS,
                                                __Visitor)
            }
        }
    };
/// Configuration for the size trigger.
pub struct SizeTriggerConfig {
    limit: u64,
}
