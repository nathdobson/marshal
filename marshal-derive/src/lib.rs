#![deny(unused_must_use)]

extern crate proc_macro;

use proc_macro2::Ident;
use syn::{DeriveInput, LitStr, parse_macro_input};

use crate::deserialize::derive_deserialize_impl;
use crate::deserialize_update::derive_deserialize_update_impl;
use crate::serialize::derive_serialize_impl;
use crate::serialize_stream::derive_serialize_stream_impl;
use crate::serialize_update::derive_serialize_update_impl;

mod deserialize;
mod deserialize_update;
mod generics;
mod parsed_enum;
mod parsed_fields;
mod serialize;
mod serialize_stream;
mod serialize_update;

#[proc_macro_derive(Deserialize)]
pub fn derive_deserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_deserialize_impl(&input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

#[proc_macro_derive(DeserializeUpdate)]
pub fn derive_deserialize_update(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_deserialize_update_impl(&input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

fn ident_to_lit(ident: &Ident) -> LitStr {
    LitStr::new(&format!("{}", ident), ident.span())
}

#[proc_macro_derive(Serialize)]
pub fn derive_serialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_serialize_impl(&input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

#[proc_macro_derive(SerializeUpdate)]
pub fn derive_serialize_update(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_serialize_update_impl(&input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

#[proc_macro_derive(SerializeStream)]
pub fn derive_serialize_stream(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_serialize_stream_impl(&input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}
