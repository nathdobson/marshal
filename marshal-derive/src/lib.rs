extern crate proc_macro;

use quote::quote;
use syn::{Data, DataStruct, DeriveInput, Fields, LitStr, parse_macro_input};
use syn::__private::TokenStream2;

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
            let parser_trait = quote!(::marshal::reexports::marshal_core::parse::Parser);

            let any_parser_trait = quote!(::marshal::reexports::marshal_core::parse::AnyParser);
            let any_parser_type = quote!(<P as #parser_trait<'de>>::AnyParser<'_>);
            let as_any_parser = quote!(<#any_parser_type as #any_parser_trait<P>>);

            let map_parser_trait = quote!(::marshal::reexports::marshal_core::parse::MapParser);
            let map_parser_type = quote!(<P as #parser_trait<'de>>::MapParser<'_>);
            let as_map_parser = quote!(<#map_parser_type as #map_parser_trait<P>>);

            let entry_parser_trait = quote!(::marshal::reexports::marshal_core::parse::EntryParser);
            let entry_parser_type = quote!(<P as #parser_trait<'de>>::EntryParser<'_>);
            let as_entry_parser = quote!(<#entry_parser_type as #entry_parser_trait<P>>);

            let deserialize_trait = quote!(::marshal::de::Deserialize);
            let result_type = quote!(::marshal::reexports::anyhow::Result);
            let context_type = quote!(::marshal::context::Context);
            let parse_hint_type = quote!(::marshal::reexports::marshal_core::parse::ParseHint);
            let parser_view_type = quote!(::marshal::reexports::marshal_core::parse::ParserView);
            let type_name = LitStr::new(&format!("{}", type_ident), type_ident.span());
            let option_type = quote! {::std::option::Option};
            let missing_field_error_type = quote! {::marshal::de::MissingFieldError};
            match fields {
                Fields::Named(fields) => {
                    let field_names = fields
                        .named
                        .iter()
                        .map(|x| x.ident.as_ref().unwrap())
                        .collect::<Vec<_>>();
                    let field_types = fields.named.iter().map(|x| &x.ty).collect::<Vec<_>>();
                    let field_name_literals = field_names
                        .iter()
                        .map(|x| LitStr::new(&format!("{}", x), type_ident.span()))
                        .collect::<Vec<_>>();
                    output = quote! {
                        impl<'de, P: #parser_trait<'de>> #deserialize_trait<'de, P> for #type_ident {
                            fn deserialize(parser: #any_parser_type, ctx: &mut #context_type) -> #result_type<Self>{
                                let hint = #parse_hint_type::Struct{
                                    fields: &[
                                        #(
                                            #field_name_literals
                                        ),*
                                    ],
                                    name: #type_name,
                                };
                                #(
                                    let #field_names : #option_type<field_types> = #option_type::None;
                                )*
                                let parser = #as_any_parser::parse(parser, hint)?;
                                match parser {
                                    #parser_view_type::Map(mut parser) => {
                                        while let Some(mut entry)=#as_map_parser::parse_next(&mut parser)?{
                                            match #as_any_parser::parse(#as_entry_parser::parse_key(&mut entry)?,#parse_hint_type::Identifier)?{
                                                _=>todo!("A")
                                            }
                                        }
                                    },
                                     _ => todo!("b"),
                                }
                                #(
                                    let #field_names = #field_names.ok_or(#missing_field_error_type{name:#field_name_literals})?;
                                )*
                                Ok(#type_ident {
                                    #(
                                        #field_names
                                    ),*
                                })
                            }
                        }
                    }
                }
                Fields::Unnamed(_) => todo!(),
                Fields::Unit => todo!(),
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
    let writer_trait = quote! { ::marshal::reexports::marshal_core::write::Writer };
    let any_writer_trait = quote! { ::marshal::reexports::marshal_core::write::AnyWriter };
    let struct_writer_trait = quote! { ::marshal::reexports::marshal_core::write::StructWriter };
    let serialize_trait = quote! { ::marshal::ser::Serialize };
    let context_type = quote! { ::marshal::context::Context };
    let type_name = LitStr::new(&format!("{}", type_ident), type_ident.span());
    let any_writer_type = quote!(<W as #writer_trait>::AnyWriter<'_>);
    let struct_writer_type = quote!(<W as #writer_trait>::StructWriter<'_>);
    let as_any_writer = quote!(<#any_writer_type as #any_writer_trait<W>>);
    let as_struct_writer = quote!(<#struct_writer_type as #struct_writer_trait<W>>);
    match data {
        Data::Struct(data) => {
            let DataStruct {
                struct_token,
                fields,
                semi_token,
            } = data;
            match fields {
                Fields::Unit => output = quote! {},
                Fields::Named(fields) => {
                    let field_count = fields.named.len();
                    let field_names = fields
                        .named
                        .iter()
                        .map(|x| x.ident.as_ref().unwrap())
                        .collect::<Vec<_>>();
                    output = quote! {
                        impl<W: #writer_trait> #serialize_trait<W> for #type_ident {
                            fn serialize(&self, writer: #any_writer_type, ctx: &mut #context_type) -> anyhow::Result<()> {
                                let mut writer = #as_any_writer::write_struct(writer, #type_name, #field_count)?;
                                #(
                                    #serialize_trait<W>::serialize(self.#field_names, #as_struct_writer::write_field(&mut writer,#field_names)?)?;
                                )*
                                #as_struct_writer::end(writer)?;
                                Ok(())
                            }
                        }
                    }
                }
                Fields::Unnamed(_) => output = quote! {},
            }
        }
        Data::Enum(_) => todo!(),
        Data::Union(_) => todo!(),
    }
    Ok(output)
}
