#[allow(dead_code)]
#[cfg(test)]
#[no_implicit_prelude]
mod test_no_prelude {
    extern crate marshal;
    #[derive(marshal::Serialize, marshal::Deserialize)]
    struct Foo;
    marshal::derive_deserialize_rc_transparent!(Foo);
    marshal::derive_deserialize_arc_transparent!(Foo);
    marshal::derive_serialize_rc_transparent!(Foo);
    marshal::derive_serialize_arc_transparent!(Foo);
}