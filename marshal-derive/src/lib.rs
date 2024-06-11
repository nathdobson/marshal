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
    let decoder_trait = quote!(::marshal::decode::Decoder);
    let primitive_type = quote!(::marshal::Primitive);

    let any_decoder_trait = quote!(::marshal::decode::AnyDecoder);
    let any_decoder_type = quote!(<P as #decoder_trait<'de>>::AnyDecoder<'_>);
    let as_any_decoder = quote!(<#any_decoder_type as #any_decoder_trait<P>>);

    let map_decoder_trait = quote!(::marshal::decode::MapDecoder);
    let map_decoder_type = quote!(<P as #decoder_trait<'de>>::MapDecoder<'_>);
    let as_map_decoder = quote!(<#map_decoder_type as #map_decoder_trait<P>>);

    let seq_decoder_trait = quote!(::marshal::decode::SeqDecoder);
    let seq_decoder_type = quote!(<P as #decoder_trait<'de>>::SeqDecoder<'_>);
    let as_seq_decoder = quote!(<#seq_decoder_type as #seq_decoder_trait<P>>);

    let entry_decoder_trait = quote!(::marshal::decode::EntryDecoder);
    let entry_decoder_type = quote!(<P as #decoder_trait<'de>>::EntryDecoder<'_>);
    let as_entry_decoder = quote!(<#entry_decoder_type as #entry_decoder_trait<P>>);

    let enum_decoder_trait = quote!(::marshal::decode::EnumDecoder);
    let enum_decoder_type = quote!(<P as #decoder_trait<'de>>::EnumDecoder<'_>);
    let as_enum_decoder = quote!(<#enum_decoder_type as #enum_decoder_trait<P>>);

    let deserialize_trait = quote!(::marshal::de::Deserialize);
    let result_type = quote!(::marshal::reexports::anyhow::Result);
    let context_type = quote!(::marshal::context::Context);
    let decode_hint_type = quote!(::marshal::decode::DecodeHint);
    let decode_variant_hint_type = quote!(::marshal::decode::DecodeVariantHint);
    let decoder_view_type = quote!(::marshal::decode::DecoderView);
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
                        impl<'de, P: #decoder_trait<'de>> #deserialize_trait<'de, P> for #type_ident {
                            #[allow(unreachable_code)]
                            fn deserialize(decoder: #any_decoder_type, ctx: &mut #context_type) -> #result_type<Self>{
                                let hint = #decode_hint_type::Struct{
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
                                let decoder = #as_any_decoder::decode(decoder, hint)?;
                                match decoder {
                                    #decoder_view_type::Map(mut decoder) => {
                                        while let Some(mut entry) = #as_map_decoder::decode_next(&mut decoder)?{
                                            let field_index: Option<usize> = match #as_any_decoder::decode(#as_entry_decoder::decode_key(&mut entry)?,#decode_hint_type::Identifier)?{
                                                #decoder_view_type::String(name) => match &*name {
                                                    #(
                                                        #field_name_literals => Some(#field_name_indexes),
                                                    )*
                                                    _ => None,
                                                },
                                                #decoder_view_type::Primitive(x) => Some(<usize as TryFrom<#primitive_type>>::try_from(x)?),
                                                v => v.mismatch("field name or index")?,
                                            };
                                            if let Some(field_index) = field_index{
                                                match field_index {
                                                    #(
                                                        #field_name_indexes => {
                                                            let value = #as_entry_decoder::decode_value(&mut entry)?;
                                                            #field_names = Some(<#field_types as #deserialize_trait<'de, P>>::deserialize(value, ctx)?);
                                                        }
                                                    )*
                                                    _ => {
                                                        #as_any_decoder::ignore(#as_entry_decoder::decode_value(&mut entry)?)?;
                                                    },
                                                }
                                            }else{
                                                #as_any_decoder::ignore(#as_entry_decoder::decode_value(&mut entry)?)?;
                                            };
                                            #as_entry_decoder::decode_end(entry)?;
                                        }
                                    },
                                    v => v.mismatch("map from field names or indices to field values")?,
                                }
                                #(
                                    let #field_names = #field_names.ok_or(#schema_error::MissingField{field_name:#field_name_literals})?;
                                )*
                                ::std::result::Result::Ok(#type_ident {
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
                        impl<'de, P: #decoder_trait<'de>> #deserialize_trait<'de, P> for #type_ident {
                            fn deserialize(decoder: #any_decoder_type, ctx: &mut #context_type) -> #result_type<Self>{
                                match #as_any_decoder::decode(decoder, #decode_hint_type::TupleStruct{name:#type_name, len:#field_count})?{
                                    #decoder_view_type::Seq(mut decoder) => {
                                        let result=#type_ident(
                                            #(
                                                {
                                                    let x = <#field_types as #deserialize_trait<'de, P> >::deserialize(
                                                        #as_seq_decoder::decode_next(&mut decoder)?
                                                            .ok_or(#schema_error::TupleTooShort)?,
                                                        ctx
                                                    )?;
                                                    x
                                                },
                                            )*
                                        );
                                        #as_seq_decoder::ignore(decoder)?;
                                        ::std::result::Result::Ok(result)
                                    },
                                    v => v.mismatch("seq")?
                                }
                            }
                        }
                    }
                }
                Fields::Unit => {
                    output = quote! {
                        impl<'de, P: #decoder_trait<'de>> #deserialize_trait<'de, P> for #type_ident {
                            fn deserialize(decoder: #any_decoder_type, ctx: &mut #context_type) -> #result_type<Self>{
                                match #as_any_decoder::decode(decoder, #decode_hint_type::UnitStruct{name:#type_name})?{
                                    #decoder_view_type::Primitive(#primitive_type::Unit) => ::std::result::Result::Ok(#type_ident),
                                    v => v.mismatch("unit")?,
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
                        let field_idents: Vec<_> = fields.named.iter().map(|x| x.ident.as_ref().unwrap()).collect();
                        let field_types: Vec<_> = fields.named.iter().map(|x| &x.ty).collect();
                        let field_names: Vec<_> = field_idents.iter().map(|x| ident_to_lit(x)).collect();
                        let field_indexes: Vec<_> = (0..fields.named.len()).collect();
                        matches.push(quote! {
                            #variant_index => {
                                let hint = #decode_variant_hint_type::StructVariant{
                                    fields: &[
                                        #(
                                            #field_names
                                        ),*
                                    ],
                                };
                                #(
                                    let mut #field_idents : #option_type<#field_types> = #option_type::None;
                                )*
                                let decoder = #as_enum_decoder::decode_variant(&mut decoder, hint)?;
                                match decoder {
                                    #decoder_view_type::Map(mut decoder) => {
                                        while let Some(mut entry) = #as_map_decoder::decode_next(&mut decoder)?{
                                            let field_index:Option<usize> = match #as_any_decoder::decode(#as_entry_decoder::decode_key(&mut entry)?,#decode_hint_type::Identifier)?{
                                                #decoder_view_type::String(name) => match &*name{
                                                    #(
                                                        #field_names => Some(#field_indexes),
                                                    )*
                                                    _ => None,
                                                },
                                                #decoder_view_type::Primitive(x) => Some(<usize as TryFrom<#primitive_type>>::try_from(x)?),
                                                v => v.mismatch("field name or index")?,
                                            };
                                            if let Some(field_index)=field_index{
                                                match field_index {
                                                    #(
                                                        #field_indexes => {
                                                            let value = #as_entry_decoder::decode_value(&mut entry)?;
                                                            #field_idents = Some(<#field_types as #deserialize_trait<'de, P>>::deserialize(value, ctx)?);
                                                        }
                                                    )*
                                                    _ => #as_any_decoder::ignore(#as_entry_decoder::decode_value(&mut entry)?)?,
                                                }
                                            }else{
                                                #as_any_decoder::ignore(#as_entry_decoder::decode_value(&mut entry)?)?;
                                            }
                                            #as_entry_decoder::decode_end(entry)?;
                                        }
                                    },
                                    v => v.mismatch("expected map")?
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
                        let field_count = fields.unnamed.len();
                        let field_types: Vec<_> = fields.unnamed.iter().map(|x| &x.ty).collect();
                        matches.push(quote! {
                            #variant_index => {
                                match #as_enum_decoder::decode_variant(&mut decoder, #decode_variant_hint_type::TupleVariant{ len: #field_count })?{
                                    #decoder_view_type::Seq(mut decoder) => {
                                        let result=#type_ident::#variant_ident(
                                            #(
                                                {
                                                    let x = <#field_types as #deserialize_trait<'de, P> >::deserialize(
                                                        #as_seq_decoder::decode_next(&mut decoder)?
                                                            .ok_or(#schema_error::TupleTooShort)?,
                                                        ctx
                                                    )?;
                                                    x
                                                },
                                            )*
                                        );
                                        #as_seq_decoder::ignore(decoder)?;
                                        result
                                    },
                                    v => v.mismatch("seq")?
                                }
                            },
                        })
                    }

                    Fields::Unit => matches.push(quote! {
                        #variant_index => {
                            let variant = #as_enum_decoder::decode_variant(&mut decoder, #decode_variant_hint_type::UnitVariant)?;
                            variant.ignore()?;
                            #type_ident::#variant_ident
                        },
                    }),
                }
            }
            output = quote! {
                impl<'de, P: #decoder_trait<'de>> #deserialize_trait<'de, P> for #type_ident {
                    fn deserialize(decoder: #any_decoder_type, ctx: &mut #context_type) -> #result_type<Self>{
                        let hint = #decode_hint_type::Enum {
                            variants: &[
                                #(
                                    #variant_names
                                ),*
                            ],
                            name: #type_name,
                        };
                        let decoder = #as_any_decoder::decode(decoder, hint)?;
                        match decoder {
                            #decoder_view_type::Enum(mut decoder) => {
                                let variant_index = {
                                    let disc = #as_enum_decoder::decode_discriminant(&mut decoder)?;
                                    let disc = #as_any_decoder::decode(disc, #decode_hint_type::Identifier)?;
                                    match disc {
                                        #decoder_view_type::Primitive(variant_index) => usize::try_from(variant_index)?,
                                        #decoder_view_type::String(disc) => match &*disc {
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
                                #as_enum_decoder::decode_end(decoder)?;
                                ::std::result::Result::Ok(result)
                            },
                            v => v.mismatch("enum")?,
                        }
                    }
                }
            };
        }
        Data::Union(u) => {
            return Err(syn::Error::new(
                u.union_token.span,
                "Cannot derive Deserialize for for unions.",
            ))?;
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
    let encoder_trait = quote! { ::marshal::encode::Encoder };
    let any_encoder_trait = quote! { ::marshal::encode::AnyEncoder };
    let struct_encoder_trait = quote! { ::marshal::encode::StructEncoder };
    let struct_variant_encoder_trait = quote! { ::marshal::encode::StructVariantEncoder };
    let tuple_variant_encoder_trait = quote! { ::marshal::encode::TupleVariantEncoder };
    let tuple_struct_encoder_trait = quote! { ::marshal::encode::TupleStructEncoder };
    let serialize_trait = quote! { ::marshal::ser::Serialize };
    let context_type = quote! { ::marshal::context::Context };
    let type_name = LitStr::new(&format!("{}", type_ident), type_ident.span());
    let any_encoder_type = quote!(<W as #encoder_trait>::AnyEncoder<'_>);

    let struct_encoder_type = quote!(<W as #encoder_trait>::StructEncoder<'_>);
    let as_any_encoder = quote!(<#any_encoder_type as #any_encoder_trait<W>>);
    let as_struct_encoder = quote!(<#struct_encoder_type as #struct_encoder_trait<W>>);

    let struct_variant_encoder_type = quote!(<W as #encoder_trait>::StructVariantEncoder<'_>);
    let as_struct_variant_encoder =
        quote!(<#struct_variant_encoder_type as #struct_variant_encoder_trait<W>>);
    let tuple_variant_encoder_type = quote!(<W as #encoder_trait>::TupleVariantEncoder<'_>);
    let as_tuple_variant_encoder =
        quote!(<#tuple_variant_encoder_type as #tuple_variant_encoder_trait<W>>);

    let tuple_struct_encoder_type = quote!(<W as #encoder_trait>::TupleStructEncoder<'_>);
    let as_tuple_struct_encoder =
        quote!(<#tuple_struct_encoder_type as #tuple_struct_encoder_trait<W>>);
    let anyhow = quote!(::marshal::reexports::anyhow);
    let result_type = quote!(#anyhow::Result);

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
                        impl<W: #encoder_trait> #serialize_trait<W> for #type_ident {
                            fn serialize(&self, encoder: #any_encoder_type, ctx: &mut #context_type) -> #result_type<()> {
                                #as_any_encoder::encode_unit_struct(encoder,#type_name)
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
                        impl<W: #encoder_trait> #serialize_trait<W> for #type_ident {
                            fn serialize(&self, encoder: #any_encoder_type, ctx: &mut #context_type) -> #result_type<()> {
                                let mut encoder = #as_any_encoder::encode_struct(encoder, #type_name, &[
                                        #(
                                            #field_name_literals
                                        ),*
                                    ])?;
                                #(
                                    #serialize_trait::<W>::serialize(&self.#field_names, #as_struct_encoder::encode_field(&mut encoder)?, ctx)?;
                                )*
                                #as_struct_encoder::end(encoder)?;
                                ::std::result::Result::Ok(())
                            }
                        }
                    }
                }
                Fields::Unnamed(fields) => {
                    let field_count = fields.unnamed.len();
                    let field_index: Vec<_> = (0..field_count).map(syn::Index::from).collect();
                    output = quote! {
                        impl<W: #encoder_trait> #serialize_trait<W> for #type_ident {
                            fn serialize(&self, encoder: #any_encoder_type, ctx: &mut #context_type) -> #result_type<()> {
                                let mut encoder=#as_any_encoder::encode_tuple_struct(encoder, #type_name, #field_count)?;
                                #(
                                    #serialize_trait::<W>::serialize(&self.#field_index, #as_tuple_struct_encoder::encode_field(&mut encoder)?, ctx)?;
                                )*
                                #as_tuple_struct_encoder::end(encoder)?;
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
                                let mut encoder = #as_any_encoder::encode_struct_variant(encoder, #type_name, &[#( #variant_names ),*], #variant_index, &[#(#field_names),*])?;
                                #(
                                    #serialize_trait::<W>::serialize(#field_idents, #as_struct_variant_encoder::encode_field(&mut encoder)?, ctx)?;
                                )*
                                #as_struct_variant_encoder::end(encoder)?;
                                ::std::result::Result::Ok(())
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
                                let mut encoder = #as_any_encoder::encode_tuple_variant(encoder, #type_name, &[#( #variant_names ),*], #variant_index, #field_count)?;
                                #(
                                    #serialize_trait::<W>::serialize(#field_idents, #as_tuple_variant_encoder::encode_field(&mut encoder)?, ctx)?;
                                )*
                                #as_tuple_variant_encoder::end(encoder)?;
                                ::std::result::Result::Ok(())
                            },
                        });
                    }
                    Fields::Unit => {
                        matches.push(quote! {
                            Self::#variant_ident => {
                                #as_any_encoder::encode_unit_variant(encoder, #type_name, &[#( #variant_names ),*], #variant_index)?;
                                ::std::result::Result::Ok(())
                            },
                        });
                    }
                }
            }
            output = quote! {
                impl<W: #encoder_trait> #serialize_trait<W> for #type_ident {
                    fn serialize(&self, encoder: #any_encoder_type, ctx: &mut #context_type) -> #result_type<()> {
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
            ))?;
        }
    }
    Ok(output)
}
