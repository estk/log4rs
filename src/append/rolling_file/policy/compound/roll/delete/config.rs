#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DESERIALIZE_FOR_DeleteRollerConfig: () =
    {
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Deserialize for DeleteRollerConfig {
            fn deserialize<__D>(deserializer: __D)
             -> _serde::export::Result<DeleteRollerConfig, __D::Error> where
             __D: _serde::Deserializer {
                #[allow(non_camel_case_types)]
                enum __Field { }
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
                                    _ =>
                                    Err(_serde::de::Error::unknown_field(value,
                                                                         FIELDS)),
                                }
                            }
                            fn visit_bytes<__E>(self, value: &[u8])
                             -> _serde::export::Result<__Field, __E> where
                             __E: _serde::de::Error {
                                match value {
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
                    DeleteRollerConfig;
                    fn expecting(&self,
                                 formatter:
                                     &mut _serde::export::fmt::Formatter)
                     -> _serde::export::fmt::Result {
                        formatter.write_str("struct DeleteRollerConfig")
                    }
                    #[inline]
                    fn visit_seq<__V>(self, _: __V)
                     -> _serde::export::Result<DeleteRollerConfig, __V::Error>
                     where __V: _serde::de::SeqVisitor {
                        let __field0 = _serde::export::Default::default();
                        Ok(DeleteRollerConfig{_p: __field0,})
                    }
                    #[inline]
                    fn visit_map<__V>(self, mut visitor: __V)
                     -> _serde::export::Result<DeleteRollerConfig, __V::Error>
                     where __V: _serde::de::MapVisitor {
                        try!(visitor . visit_key :: < __Field > (
                             )).map(|impossible| match impossible { });
                        Ok(DeleteRollerConfig{_p:
                                                  _serde::export::Default::default(),})
                    }
                }
                const FIELDS: &'static [&'static str] = &[];
                deserializer.deserialize_struct("DeleteRollerConfig", FIELDS,
                                                __Visitor)
            }
        }
    };
/// Configuration for the delete roller.
pub struct DeleteRollerConfig {
    _p: (),
}
