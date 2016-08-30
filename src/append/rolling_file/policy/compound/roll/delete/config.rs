#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DESERIALIZE_FOR_DeleteRollerConfig: () =
    {
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::de::Deserialize for DeleteRollerConfig {
            fn deserialize<__D>(deserializer: &mut __D)
             -> ::std::result::Result<DeleteRollerConfig, __D::Error> where
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
                                    "_p" => { Ok(__Field::__field0) }
                                    _ =>
                                    Err(_serde::de::Error::unknown_field(value)),
                                }
                            }
                            fn visit_bytes<__E>(&mut self, value: &[u8])
                             -> ::std::result::Result<__Field, __E> where
                             __E: _serde::de::Error {
                                match value {
                                    b"_p" => { Ok(__Field::__field0) }
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
                    DeleteRollerConfig;
                    #[inline]
                    fn visit_seq<__V>(&mut self, mut visitor: __V)
                     -> ::std::result::Result<DeleteRollerConfig, __V::Error>
                     where __V: _serde::de::SeqVisitor {
                        let __field0 = ::std::default::Default::default();
                        try!(visitor . end (  ));
                        Ok(DeleteRollerConfig{_p: __field0,})
                    }
                    #[inline]
                    fn visit_map<__V>(&mut self, mut visitor: __V)
                     -> ::std::result::Result<DeleteRollerConfig, __V::Error>
                     where __V: _serde::de::MapVisitor {
                        while let Some(key) =
                                  try!(visitor . visit_key :: < __Field > (
                                       )) {
                            match key {
                                __Field::__field0 => {
                                    try!(visitor . visit_value :: < _serde ::
                                         de :: impls :: IgnoredAny > (  ));
                                }
                            }
                        }
                        try!(visitor . end (  ));
                        Ok(DeleteRollerConfig{_p:
                                                  ::std::default::Default::default(),})
                    }
                }
                const FIELDS: &'static [&'static str] = &["_p"];
                deserializer.deserialize_struct("DeleteRollerConfig", FIELDS,
                                                __Visitor)
            }
        }
    };
/// Configuration for the delete roller.
pub struct DeleteRollerConfig {
    _p: (),
}
