#![deny(unused_must_use)]
#![deny(warnings)]

extern crate proc_macro;

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::__private::TokenStream2;
use syn::{parse_macro_input, Data, DataEnum, DataStruct, DeriveInput, Fields, LitStr, Variant};

#[proc_macro_derive(Deserialize)]
pub fn derive_deserialize(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    derive_deserialize_impl(&input)
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

fn ident_to_lit(ident: &Ident) -> LitStr {
    LitStr::new(&format!("{}", ident), ident.span())
}

fn derive_deserialize_impl(input: &DeriveInput) -> Result<TokenStream2, syn::Error> {
    let DeriveInput {
        attrs: _,
        vis: _,
        ident: type_ident,
        generics: _,
        data,
    } = input;
    let output: TokenStream2;
    let parser_trait = quote!(::marshal::reexports::marshal_core::decode::Parser);
    let primitive_type = quote!(::marshal::reexports::marshal_core::Primitive);

    let any_parser_trait = quote!(::marshal::reexports::marshal_core::decode::AnyParser);
    let any_parser_type = quote!(<P as #parser_trait<'de>>::AnyParser<'_>);
    let as_any_parser = quote!(<#any_parser_type as #any_parser_trait<P>>);

    let map_parser_trait = quote!(::marshal::reexports::marshal_core::decode::MapParser);
    let map_parser_type = quote!(<P as #parser_trait<'de>>::MapParser<'_>);
    let as_map_parser = quote!(<#map_parser_type as #map_parser_trait<P>>);

    let seq_parser_trait = quote!(::marshal::reexports::marshal_core::decode::SeqParser);
    let seq_parser_type = quote!(<P as #parser_trait<'de>>::SeqParser<'_>);
    let as_seq_parser = quote!(<#seq_parser_type as #seq_parser_trait<P>>);

    let entry_parser_trait = quote!(::marshal::reexports::marshal_core::decode::EntryParser);
    let entry_parser_type = quote!(<P as #parser_trait<'de>>::EntryParser<'_>);
    let as_entry_parser = quote!(<#entry_parser_type as #entry_parser_trait<P>>);

    let enum_parser_trait = quote!(::marshal::reexports::marshal_core::decode::EnumParser);
    let enum_parser_type = quote!(<P as #parser_trait<'de>>::EnumParser<'_>);
    let as_enum_parser = quote!(<#enum_parser_type as #enum_parser_trait<P>>);

    let deserialize_trait = quote!(::marshal::de::Deserialize);
    let result_type = quote!(::marshal::reexports::anyhow::Result);
    let context_type = quote!(::marshal::context::Context);
    let parse_hint_type = quote!(::marshal::reexports::marshal_core::decode::ParseHint);
    let parse_variant_hint_type =
        quote!(::marshal::reexports::marshal_core::decode::ParseVariantHint);
    let parser_view_type = quote!(::marshal::reexports::marshal_core::decode::ParserView);
    let type_name = ident_to_lit(&type_ident);
    let option_type = quote! {::std::option::Option};
    let schema_error = quote! {::marshal::de::SchemaError};
    match data {
        Data::Struct(data) => {
            let DataStruct {
                struct_token: _,
                fields,
                semi_token: _,
            } = data;

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
                    let field_name_indexes = (0..fields.named.len()).collect::<Vec<_>>();
                    output = quote! {
                        impl<'de, P: #parser_trait<'de>> #deserialize_trait<'de, P> for #type_ident {
                            #[allow(unreachable_code)]
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
                                    let mut #field_names : #option_type<#field_types> = #option_type::None;
                                )*
                                let parser = #as_any_parser::parse(parser, hint)?;
                                match parser {
                                    #parser_view_type::Map(mut parser) => {
                                        while let Some(mut entry) = #as_map_parser::parse_next(&mut parser)?{
                                            let field_index:usize = match #as_any_parser::parse(#as_entry_parser::parse_key(&mut entry)?,#parse_hint_type::Identifier)?{
                                                #parser_view_type::String(name) => match &*name{
                                                    #(
                                                        #field_name_literals => #field_name_indexes,
                                                    )*
                                                    _ => todo!("unexpected field name"),
                                                },
                                                #parser_view_type::Primitive(x) => <usize as TryFrom<#primitive_type>>::try_from(x)?,
                                                _=> todo!("unexpected type instead of field name or index")
                                            };
                                            match field_index {
                                                #(
                                                    #field_name_indexes => {
                                                        let value = #as_entry_parser::parse_value(&mut entry)?;
                                                        #field_names = Some(<#field_types as #deserialize_trait<'de, P>>::deserialize(value, ctx)?);
                                                    }
                                                )*
                                                _=>todo!("unknown field index"),
                                            }
                                            #as_entry_parser::parse_end(entry)?;
                                        }
                                    },
                                     _ => todo!("expected map"),
                                }
                                #(
                                    let #field_names = #field_names.ok_or(#schema_error::MissingField{field_name:#field_name_literals})?;
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
                Fields::Unnamed(fields) => {
                    let field_count = fields.unnamed.len();
                    let field_types = fields.unnamed.iter().map(|x| &x.ty).collect::<Vec<_>>();
                    output = quote! {
                        impl<'de, P: #parser_trait<'de>> #deserialize_trait<'de, P> for #type_ident {
                            fn deserialize(parser: #any_parser_type, ctx: &mut #context_type) -> #result_type<Self>{
                                match #as_any_parser::parse(parser, #parse_hint_type::TupleStruct{name:#type_name, len:#field_count})?{
                                    #parser_view_type::Seq(mut parser) => {
                                        let result=#type_ident(
                                            #(
                                                {
                                                    let x = <#field_types as #deserialize_trait<'de, P> >::deserialize(
                                                        #as_seq_parser::parse_next(&mut parser)?
                                                            .ok_or(#schema_error::TupleTooShort)?,
                                                        ctx
                                                    )?;
                                                    x
                                                },
                                            )*
                                        );
                                        #as_seq_parser::ignore(parser)?;
                                        Ok(result)
                                    },
                                    _ => todo!("b")
                                }
                            }
                        }
                    }
                }
                Fields::Unit => {
                    output = quote! {
                        impl<'de, P: #parser_trait<'de>> #deserialize_trait<'de, P> for #type_ident {
                            fn deserialize(parser: #any_parser_type, ctx: &mut #context_type) -> #result_type<Self>{
                                match #as_any_parser::parse(parser, #parse_hint_type::UnitStruct{name:#type_name})?{
                                    #parser_view_type::Primitive(#primitive_type::Unit) => Ok(#type_ident),
                                    _ => todo!("expected unit"),
                                }
                            }
                        }
                    }
                }
            }
        }
        Data::Enum(data) => {
            let DataEnum {
                enum_token: _,
                brace_token: _,
                variants,
            } = data;
            let mut variant_names: Vec<LitStr> = vec![];
            let variant_indexes: Vec<usize> = (0..variants.len()).collect();
            for variant in variants {
                variant_names.push(ident_to_lit(&variant.ident));
            }
            let mut matches: Vec<TokenStream> = vec![];
            for (variant_index, variant) in variants.iter().enumerate() {
                let Variant {
                    attrs: _,
                    ident: variant_ident,
                    fields,
                    discriminant: _,
                } = variant;
                match &fields {
                    Fields::Named(fields) => {
                        let field_idents:Vec<_>=fields.named.iter().map(|x|x.ident.as_ref().unwrap()).collect();
                        let field_types:Vec<_> =fields.named.iter().map(|x|&x.ty).collect();
                        let field_names:Vec<_> = field_idents.iter().map(|x|ident_to_lit(x)).collect();
                        let field_indexes:Vec<_> =(0..fields.named.len()).collect();
                        matches.push(quote! {
                            #variant_index => {
                                let hint = #parse_variant_hint_type::StructVariant{
                                    fields: &[
                                        #(
                                            #field_names
                                        ),*
                                    ],
                                };
                                #(
                                    let mut #field_idents : #option_type<#field_types> = #option_type::None;
                                )*
                                let parser = #as_enum_parser::parse_variant(&mut parser, hint)?;
                                match parser {
                                    #parser_view_type::Map(mut parser) => {
                                        while let Some(mut entry) = #as_map_parser::parse_next(&mut parser)?{
                                            let field_index:usize = match #as_any_parser::parse(#as_entry_parser::parse_key(&mut entry)?,#parse_hint_type::Identifier)?{
                                                #parser_view_type::String(name) => match &*name{
                                                    #(
                                                        #field_names => #field_indexes,
                                                    )*
                                                    _ => todo!("unexpected field name"),
                                                },
                                                #parser_view_type::Primitive(x) => <usize as TryFrom<#primitive_type>>::try_from(x)?,
                                                _=> todo!("unexpected type instead of field name or index")
                                            };
                                            match field_index {
                                                #(
                                                    #field_indexes => {
                                                        let value = #as_entry_parser::parse_value(&mut entry)?;
                                                        #field_idents = Some(<#field_types as #deserialize_trait<'de, P>>::deserialize(value, ctx)?);
                                                    }
                                                )*
                                                _=>todo!("unknown field index"),
                                            }
                                            #as_entry_parser::parse_end(entry)?;
                                        }
                                    },
                                     _ => todo!("expected map"),
                                }
                                #(
                                    let #field_idents = #field_idents.ok_or(#schema_error::MissingField{field_name:#field_names})?;
                                )*
                                #type_ident::#variant_ident {
                                    #(
                                        #field_idents
                                    ),*
                                }
                            },
                        });
                    }

                    Fields::Unnamed(fields) => {
                        let field_count=fields.unnamed.len();
                        let field_types:Vec<_> =fields.unnamed.iter().map(|x|&x.ty).collect();
                        matches.push(quote! {
                            #variant_index => {
                                match #as_enum_parser::parse_variant(&mut parser, #parse_variant_hint_type::TupleVariant{ len: #field_count })?{
                                    #parser_view_type::Seq(mut parser) => {
                                        let result=#type_ident::#variant_ident(
                                            #(
                                                {
                                                    let x = <#field_types as #deserialize_trait<'de, P> >::deserialize(
                                                        #as_seq_parser::parse_next(&mut parser)?
                                                            .ok_or(#schema_error::TupleTooShort)?,
                                                        ctx
                                                    )?;
                                                    x
                                                },
                                            )*
                                        );
                                        #as_seq_parser::ignore(parser)?;
                                        result
                                    },
                                    _ => todo!("b")
                                }
                            },
                        })
                    },

                    Fields::Unit => matches.push(quote! {
                        #variant_index => {
                            let variant = #as_enum_parser::parse_variant(&mut parser, #parse_variant_hint_type::UnitVariant)?;
                            variant.ignore()?;
                            #type_ident::#variant_ident
                        },
                    }),
                }
            }
            output = quote! {
                impl<'de, P: #parser_trait<'de>> #deserialize_trait<'de, P> for #type_ident {
                    fn deserialize(parser: #any_parser_type, ctx: &mut #context_type) -> #result_type<Self>{
                        let hint = #parse_hint_type::Enum {
                            variants: &[
                                #(
                                    #variant_names
                                ),*
                            ],
                            name: #type_name,
                        };
                        let parser = #as_any_parser::parse(parser, hint)?;
                        match parser {
                            #parser_view_type::Enum(mut parser) => {
                                let variant_index = {
                                    let disc = #as_enum_parser::parse_discriminant(&mut parser)?;
                                    let disc = #as_any_parser::parse(disc, #parse_hint_type::Identifier)?;
                                    match disc {
                                        #parser_view_type::Primitive(variant_index) => usize::try_from(variant_index)?,
                                        #parser_view_type::String(disc) => match &*disc {
                                            #(
                                                #variant_names => #variant_indexes,
                                            )*
                                            _ => return #result_type::Err(#schema_error::UnknownVariant.into()),
                                        },
                                        _ => return #result_type::Err(#schema_error::UnknownVariant.into()),
                                    }
                                };
                                let result=match variant_index {
                                    #(#matches)*
                                    _ => return #result_type::Err(#schema_error::UnknownVariant.into()),
                                };
                                #as_enum_parser::parse_end(parser)?;
                                Ok(result)
                            },
                            _ => todo!("expected enum, but got something else"),
                        }
                    }
                }
            };
        }
        Data::Union(u) => {
            return Err(syn::Error::new(
                u.union_token.span,
                "Cannot derive Deserialize for for unions.",
            ))?
        }
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
        attrs: _,
        vis: _,
        ident: type_ident,
        generics: _,
        data,
    } = input;
    let output: TokenStream2;
    let writer_trait = quote! { ::marshal::reexports::marshal_core::encode::Writer };
    let any_writer_trait = quote! { ::marshal::reexports::marshal_core::encode::AnyWriter };
    let struct_writer_trait = quote! { ::marshal::reexports::marshal_core::encode::StructWriter };
    let struct_variant_writer_trait =
        quote! { ::marshal::reexports::marshal_core::encode::StructVariantWriter };
    let tuple_variant_writer_trait =
        quote! { ::marshal::reexports::marshal_core::encode::TupleVariantWriter };
    let tuple_struct_writer_trait =
        quote! { ::marshal::reexports::marshal_core::encode::TupleStructWriter };
    let serialize_trait = quote! { ::marshal::ser::Serialize };
    let context_type = quote! { ::marshal::context::Context };
    let type_name = LitStr::new(&format!("{}", type_ident), type_ident.span());
    let any_writer_type = quote!(<W as #writer_trait>::AnyWriter<'_>);

    let struct_writer_type = quote!(<W as #writer_trait>::StructWriter<'_>);
    let as_any_writer = quote!(<#any_writer_type as #any_writer_trait<W>>);
    let as_struct_writer = quote!(<#struct_writer_type as #struct_writer_trait<W>>);

    let struct_variant_writer_type = quote!(<W as #writer_trait>::StructVariantWriter<'_>);
    let as_struct_variant_writer =
        quote!(<#struct_variant_writer_type as #struct_variant_writer_trait<W>>);
    let tuple_variant_writer_type = quote!(<W as #writer_trait>::TupleVariantWriter<'_>);
    let as_tuple_variant_writer =
        quote!(<#tuple_variant_writer_type as #tuple_variant_writer_trait<W>>);

    let tuple_struct_writer_type = quote!(<W as #writer_trait>::TupleStructWriter<'_>);
    let as_tuple_struct_writer =
        quote!(<#tuple_struct_writer_type as #tuple_struct_writer_trait<W>>);
    let result_type = quote!(::marshal::reexports::anyhow::Result);

    match data {
        Data::Struct(data) => {
            let DataStruct {
                struct_token: _,
                fields,
                semi_token: _,
            } = data;
            match fields {
                Fields::Unit => {
                    output = quote! {
                        impl<W: #writer_trait> #serialize_trait<W> for #type_ident {
                            fn serialize(&self, writer: #any_writer_type, ctx: &mut #context_type) -> anyhow::Result<()> {
                                #as_any_writer::write_unit_struct(writer,#type_name)
                            }
                        }
                    }
                }
                Fields::Named(fields) => {
                    let field_names = fields
                        .named
                        .iter()
                        .map(|x| x.ident.as_ref().unwrap())
                        .collect::<Vec<_>>();
                    let field_name_literals = field_names
                        .iter()
                        .map(|x| LitStr::new(&format!("{}", x), type_ident.span()))
                        .collect::<Vec<_>>();

                    output = quote! {
                        impl<W: #writer_trait> #serialize_trait<W> for #type_ident {
                            fn serialize(&self, writer: #any_writer_type, ctx: &mut #context_type) -> anyhow::Result<()> {
                                let mut writer = #as_any_writer::write_struct(writer, #type_name, &[
                                        #(
                                            #field_name_literals
                                        ),*
                                    ])?;
                                #(
                                    #serialize_trait::<W>::serialize(&self.#field_names, #as_struct_writer::write_field(&mut writer)?, ctx)?;
                                )*
                                #as_struct_writer::end(writer)?;
                                Ok(())
                            }
                        }
                    }
                }
                Fields::Unnamed(fields) => {
                    let field_count = fields.unnamed.len();
                    let field_index: Vec<_> = (0..field_count).map(syn::Index::from).collect();
                    output = quote! {
                        impl<W: #writer_trait> #serialize_trait<W> for #type_ident {
                            fn serialize(&self, writer: #any_writer_type, ctx: &mut #context_type) -> anyhow::Result<()> {
                                let mut writer=#as_any_writer::write_tuple_struct(writer, #type_name, #field_count)?;
                                #(
                                    #serialize_trait::<W>::serialize(&self.#field_index, #as_tuple_struct_writer::write_field(&mut writer)?, ctx)?;
                                )*
                                #as_tuple_struct_writer::end(writer)?;
                                #result_type::Ok(())
                            }
                        }
                    }
                }
            }
        }
        Data::Enum(data) => {
            let DataEnum {
                enum_token: _,
                brace_token: _,
                variants,
            } = data;
            let mut matches = vec![];
            let variant_names: Vec<_> = variants.iter().map(|x| ident_to_lit(&x.ident)).collect();
            for (variant_index, variant) in variants.iter().enumerate() {
                let Variant {
                    attrs: _,
                    ident: variant_ident,
                    fields: _,
                    discriminant: _,
                } = variant;
                let variant_index = variant_index as u32;
                match &variant.fields {
                    Fields::Named(fields) => {
                        let field_idents: Vec<_> = fields
                            .named
                            .iter()
                            .map(|x| x.ident.as_ref().unwrap())
                            .collect();
                        let field_names: Vec<_> =
                            field_idents.iter().map(|x| ident_to_lit(x)).collect();
                        matches.push(quote! {
                            Self::#variant_ident{ #(#field_idents),* } => {
                                let mut writer = #as_any_writer::write_struct_variant(writer, #type_name, &[#( #variant_names ),*], #variant_index, &[#(#field_names),*])?;
                                #(
                                    #serialize_trait::<W>::serialize(#field_idents, #as_struct_variant_writer::write_field(&mut writer)?, ctx)?;
                                )*
                                #as_struct_variant_writer::end(writer)?;
                                Ok(())
                            },
                        });
                    }
                    Fields::Unnamed(fields) => {
                        let field_count = fields.unnamed.len();
                        let field_idents: Vec<_> = (0..field_count)
                            .map(|x| format_ident!("field_{}", x))
                            .collect();
                        matches.push(quote! {
                            Self::#variant_ident(#( #field_idents ),*) => {
                                let mut writer = #as_any_writer::write_tuple_variant(writer, #type_name, &[#( #variant_names ),*], #variant_index, #field_count)?;
                                #(
                                    #serialize_trait::<W>::serialize(#field_idents, #as_tuple_variant_writer::write_field(&mut writer)?, ctx)?;
                                )*
                                #as_tuple_variant_writer::end(writer)?;
                                Ok(())
                            },
                        });
                    }
                    Fields::Unit => {
                        matches.push(quote!{
                            Self::#variant_ident => {
                                #as_any_writer::write_unit_variant(writer, #type_name, &[#( #variant_names ),*], #variant_index)?;
                                Ok(())
                            },
                        });
                    }
                }
            }
            output = quote! {
                impl<W: #writer_trait> #serialize_trait<W> for #type_ident {
                    fn serialize(&self, writer: #any_writer_type, ctx: &mut #context_type) -> anyhow::Result<()> {
                        match self{
                            #(
                                #matches
                            )*
                        }
                    }
                }
            };
        }
        Data::Union(u) => {
            return Err(syn::Error::new(
                u.union_token.span,
                "Cannot derive Deserialize for for unions.",
            ))?
        }
    }
    Ok(output)
}
