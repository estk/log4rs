#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _IMPL_DESERIALIZE_FOR_CompoundPolicyConfig: () =
    {
        extern crate serde as _serde;
        #[automatically_derived]
        impl _serde::de::Deserialize for CompoundPolicyConfig {
            fn deserialize<__D>(deserializer: &mut __D)
             -> ::std::result::Result<CompoundPolicyConfig, __D::Error> where
             __D: _serde::de::Deserializer {
                #[allow(non_camel_case_types)]
                enum __Field { __field0, __field1, }
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
                                    1usize => { Ok(__Field::__field1) }
                                    _ =>
                                    Err(_serde::de::Error::invalid_value("expected a field")),
                                }
                            }
                            fn visit_str<__E>(&mut self, value: &str)
                             -> ::std::result::Result<__Field, __E> where
                             __E: _serde::de::Error {
                                match value {
                                    "trigger" => { Ok(__Field::__field0) }
                                    "roller" => { Ok(__Field::__field1) }
                                    _ =>
                                    Err(_serde::de::Error::unknown_field(value)),
                                }
                            }
                            fn visit_bytes<__E>(&mut self, value: &[u8])
                             -> ::std::result::Result<__Field, __E> where
                             __E: _serde::de::Error {
                                match value {
                                    b"trigger" => { Ok(__Field::__field0) }
                                    b"roller" => { Ok(__Field::__field1) }
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
                    CompoundPolicyConfig;
                    #[inline]
                    fn visit_seq<__V>(&mut self, mut visitor: __V)
                     ->
                         ::std::result::Result<CompoundPolicyConfig,
                                               __V::Error> where
                     __V: _serde::de::SeqVisitor {
                        let __field0 =
                            match try!(visitor . visit :: < Trigger > (  )) {
                                Some(value) => { value }
                                None => {
                                    try!(visitor . end (  ));
                                    return Err(_serde::de::Error::invalid_length(0usize));
                                }
                            };
                        let __field1 =
                            match try!(visitor . visit :: < Roller > (  )) {
                                Some(value) => { value }
                                None => {
                                    try!(visitor . end (  ));
                                    return Err(_serde::de::Error::invalid_length(1usize));
                                }
                            };
                        try!(visitor . end (  ));
                        Ok(CompoundPolicyConfig{trigger: __field0,
                                                roller: __field1,})
                    }
                    #[inline]
                    fn visit_map<__V>(&mut self, mut visitor: __V)
                     ->
                         ::std::result::Result<CompoundPolicyConfig,
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
                        try!(visitor . end (  ));
                        let __field0 =
                            match __field0 {
                                Some(__field0) => __field0,
                                None =>
                                try!(visitor . missing_field ( "trigger" )),
                            };
                        let __field1 =
                            match __field1 {
                                Some(__field1) => __field1,
                                None =>
                                try!(visitor . missing_field ( "roller" )),
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
