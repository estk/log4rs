#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DESERIALIZE_FOR_SizeTriggerConfig: () =
    {
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::de::Deserialize for SizeTriggerConfig {
            fn deserialize<__D>(deserializer: &mut __D)
             -> ::std::result::Result<SizeTriggerConfig, __D::Error> where
             __D: _serde::de::Deserializer {
                #[allow(non_camel_case_types)]
                enum __Field { __field0, }
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
                                    _ =>
                                    Err(_serde::de::Error::invalid_value("expected a field")),
                                }
                            }
                            fn visit_str<__E>(&mut self, value: &str)
                             -> ::std::result::Result<__Field, __E> where
                             __E: _serde::de::Error {
                                match value {
                                    "limit" => { Ok(__Field::__field0) }
                                    _ =>
                                    Err(_serde::de::Error::unknown_field(value)),
                                }
                            }
                            fn visit_bytes<__E>(&mut self, value: &[u8])
                             -> ::std::result::Result<__Field, __E> where
                             __E: _serde::de::Error {
                                match value {
                                    b"limit" => { Ok(__Field::__field0) }
                                    _ => {
                                        let value =
                                            ::std::string::String::from_utf8_lossy(value);
                                        Err(_serde::de::Error::unknown_field(&value))
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
                    #[inline]
                    fn visit_seq<__V>(&mut self, mut visitor: __V)
                     -> ::std::result::Result<SizeTriggerConfig, __V::Error>
                     where __V: _serde::de::SeqVisitor {
                        let __field0 =
                            match {
                                      struct __SerdeDeserializeWithStruct {
                                          value: u64,
                                          phantom: ::std::marker::PhantomData<SizeTriggerConfig>,
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
                                                  try!(deserialize_limit ( __d
                                                       ));
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
                        Ok(SizeTriggerConfig{limit: __field0,})
                    }
                    #[inline]
                    fn visit_map<__V>(&mut self, mut visitor: __V)
                     -> ::std::result::Result<SizeTriggerConfig, __V::Error>
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
                                        Some(({
                                                  struct __SerdeDeserializeWithStruct {
                                                      value: u64,
                                                      phantom: ::std::marker::PhantomData<SizeTriggerConfig>,
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
                                                              try!(deserialize_limit
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
                            }
                        }
                        try!(visitor . end (  ));
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
