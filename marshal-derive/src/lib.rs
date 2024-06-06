extern crate proc_macro;

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::error::Error;
use syn::__private::TokenStream2;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields, Type};

#[proc_macro_derive(Deserialize)]
pub fn derive_deserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_deserialize_impl(&input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

fn derive_deserialize_impl(input: &DeriveInput) -> Result<TokenStream2, syn::Error> {
    let DeriveInput {
        attrs: type_attrs,
        vis: type_vis,
        ident: type_ident,
        generics: type_generics,
        data,
    } = input;
    let output: TokenStream2;
    match data {
        Data::Struct(data) => {
            let DataStruct {
                struct_token,
                fields,
                semi_token,
            } = data;
            let parser_trait = quote!( ::marshal::reexports::marshal_core::parse::Parser );
            let deserialize_trait = quote!(::marshal::de::Deserialize);
            let result_type = quote!(::marshal::reexports::anyhow::Result);
            let context_type = quote! { ::marshal::context::Context };
            output = quote! {
                impl<'de, P: #parser_trait<'de>> #deserialize_trait<'de, P> for #type_ident {
                    fn deserialize(parser: <P as #parser_trait<'de>>::AnyParser<'_>, ctx: &mut #context_type) -> #result_type<Self>{
                        todo!();
                    }
                }
            }
        }
        Data::Enum(_) => todo!(),
        Data::Union(_) => todo!(),
    }
    Ok(output)
}

#[proc_macro_derive(Serialize)]
pub fn derive_serialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    crate::derive_serialize_impl(&input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

fn derive_serialize_impl(input: &DeriveInput) -> Result<TokenStream2, syn::Error> {
    let DeriveInput {
        attrs: type_attrs,
        vis: type_vis,
        ident: type_ident,
        generics: type_generics,
        data,
    } = input;
    let output: TokenStream2;
    match data {
        Data::Struct(data) => {
            let DataStruct {
                struct_token,
                fields,
                semi_token,
            } = data;
            let writer_trait = quote! { ::marshal::reexports::marshal_core::write::Writer };
            let serialize_trait = quote! { ::marshal::ser::Serialize };
            let context_type = quote! { ::marshal::context::Context };
            output = quote! {
                impl<W: #writer_trait> #serialize_trait<W> for #type_ident {
                    fn serialize(&self, W: <W as #writer_trait>::AnyWriter<'_>, ctx: &mut #context_type) -> anyhow::Result<()> {
                        todo!();
                    }
                }
            }
        }
        Data::Enum(_) => todo!(),
        Data::Union(_) => todo!(),
    }
    Ok(output)
}
