#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DESERIALIZE_FOR_CompoundPolicyConfig: () =
    {
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::Deserialize for CompoundPolicyConfig {
            fn deserialize<__D>(deserializer: __D)
             -> _serde::export::Result<CompoundPolicyConfig, __D::Error> where
             __D: _serde::Deserializer {
                #[allow(non_camel_case_types)]
                enum __Field { __field0, __field1, }
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
                                    "trigger" => Ok(__Field::__field0),
                                    "roller" => Ok(__Field::__field1),
                                    _ =>
                                    Err(_serde::de::Error::unknown_field(value,
                                                                         FIELDS)),
                                }
                            }
                            fn visit_bytes<__E>(self, value: &[u8])
                             -> _serde::export::Result<__Field, __E> where
                             __E: _serde::de::Error {
                                match value {
                                    b"trigger" => Ok(__Field::__field0),
                                    b"roller" => Ok(__Field::__field1),
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
                    CompoundPolicyConfig;
                    fn expecting(&self,
                                 formatter:
                                     &mut _serde::export::fmt::Formatter)
                     -> _serde::export::fmt::Result {
                        formatter.write_str("struct CompoundPolicyConfig")
                    }
                    #[inline]
                    fn visit_seq<__V>(self, mut visitor: __V)
                     ->
                         _serde::export::Result<CompoundPolicyConfig,
                                                __V::Error> where
                     __V: _serde::de::SeqVisitor {
                        let __field0 =
                            match try!(visitor . visit :: < Trigger > (  )) {
                                Some(value) => { value }
                                None => {
                                    return Err(_serde::de::Error::invalid_length(0usize,
                                                                                 &"tuple of 2 elements"));
                                }
                            };
                        let __field1 =
                            match try!(visitor . visit :: < Roller > (  )) {
                                Some(value) => { value }
                                None => {
                                    return Err(_serde::de::Error::invalid_length(1usize,
                                                                                 &"tuple of 2 elements"));
                                }
                            };
                        Ok(CompoundPolicyConfig{trigger: __field0,
                                                roller: __field1,})
                    }
                    #[inline]
                    fn visit_map<__V>(self, mut visitor: __V)
                     ->
                         _serde::export::Result<CompoundPolicyConfig,
                                                __V::Error> where
                     __V: _serde::de::MapVisitor {
                        let mut __field0: Option<Trigger> = None;
                        let mut __field1: Option<Roller> = None;
                        while let Some(key) =
                                  try!(visitor . visit_key :: < __Field > (
                                       )) {
                            match key {
                                __Field::__field0 => {
                                    if __field0.is_some() {
                                        return Err(<__V::Error as
                                                       _serde::de::Error>::duplicate_field("trigger"));
                                    }
                                    __field0 =
                                        Some(try!(visitor . visit_value :: <
                                                  Trigger > (  )));
                                }
                                __Field::__field1 => {
                                    if __field1.is_some() {
                                        return Err(<__V::Error as
                                                       _serde::de::Error>::duplicate_field("roller"));
                                    }
                                    __field1 =
                                        Some(try!(visitor . visit_value :: <
                                                  Roller > (  )));
                                }
                            }
                        }
                        let __field0 =
                            match __field0 {
                                Some(__field0) => __field0,
                                None =>
                                try!(_serde :: de :: private :: missing_field
                                     ( "trigger" )),
                            };
                        let __field1 =
                            match __field1 {
                                Some(__field1) => __field1,
                                None =>
                                try!(_serde :: de :: private :: missing_field
                                     ( "roller" )),
                            };
                        Ok(CompoundPolicyConfig{trigger: __field0,
                                                roller: __field1,})
                    }
                }
                const FIELDS: &'static [&'static str] =
                    &["trigger", "roller"];
                deserializer.deserialize_struct("CompoundPolicyConfig",
                                                FIELDS, __Visitor)
            }
        }
    };
/// Configuration for the compound policy.
pub struct CompoundPolicyConfig {
    trigger: Trigger,
    roller: Roller,
}
