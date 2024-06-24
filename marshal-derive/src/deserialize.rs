use crate::generics::DeriveGenerics;
use crate::ident_to_lit;
use crate::parsed_enum::ParsedEnum;
use crate::parsed_fields::{ParsedFields, ParsedFieldsNamed, ParsedFieldsUnnamed};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DataEnum, DataStruct, DeriveInput, Fields, LitStr, Variant};

pub fn derive_deserialize_impl(input: &DeriveInput) -> Result<TokenStream, syn::Error> {
    let DeriveInput {
        attrs: _,
        vis: _,
        ident: type_ident,
        generics,
        data,
    } = input;
    let DeriveGenerics {
        generic_params,
        generic_args,
    } = DeriveGenerics::new(generics, &quote! {::marshal::de::Deserialize<'de,P>});
    let decoder_trait = quote!(::marshal::decode::Decoder);
    let primitive_type = quote!(::marshal::Primitive);

    let any_decoder_type = quote!(::marshal::decode::AnyDecoder);

    let deserialize_trait = quote!(::marshal::de::Deserialize);
    let result_type = quote!(::marshal::reexports::anyhow::Result);
    let context_type = quote!(::marshal::context::Context);
    let decode_hint_type = quote!(::marshal::decode::DecodeHint);
    let decode_variant_hint_type = quote!(::marshal::decode::DecodeVariantHint);
    let decoder_view_type = quote!(::marshal::decode::DecoderView);
    let type_name = ident_to_lit(&type_ident);
    let option_type = quote! {::std::option::Option};
    let schema_error = quote! {::marshal::de::SchemaError};

    let imp = quote! { impl<'de, #(#generic_params,)* P:#decoder_trait<'de>> #deserialize_trait<'de, P> for #type_ident <#(#generic_args),*> };

    match data {
        Data::Struct(data) => match ParsedFields::new(&data.fields) {
            ParsedFields::Named(ParsedFieldsNamed {
                field_idents,
                field_types,
                field_literals,
                field_indices,
            }) => Ok(quote! {
                #imp {
                    #[allow(unreachable_code)]
                    fn deserialize(decoder: #any_decoder_type<'_,'de,P>, ctx: &mut #context_type) -> #result_type<Self>{
                        let hint = #decode_hint_type::Struct{
                            fields: &[
                                #(
                                    #field_literals
                                ),*
                            ],
                            name: #type_name,
                        };
                        #(
                            let mut #field_idents : #option_type<#field_types> = #option_type::None;
                        )*
                        let decoder = decoder.decode( hint)?;
                        match decoder {
                            #decoder_view_type::Map(mut decoder) => {
                                while let Some(mut entry) = decoder.decode_next()?{
                                    let field_index: Option<usize> = match entry.decode_key()?.decode(#decode_hint_type::Identifier)?{
                                        #decoder_view_type::String(name) => match &*name {
                                            #(
                                                #field_literals => Some(#field_indices),
                                            )*
                                            _ => None,
                                        },
                                        #decoder_view_type::Primitive(x) => Some(<usize as TryFrom<#primitive_type>>::try_from(x)?),
                                        v => v.mismatch("field name or index")?,
                                    };
                                    if let Some(field_index) = field_index{
                                        match field_index {
                                            #(
                                                #field_indices => {
                                                    let value = entry.decode_value()?;
                                                    #field_idents = Some(<#field_types as #deserialize_trait<'de, P>>::deserialize(value, ctx)?);
                                                }
                                            )*
                                            _ => {
                                                entry.decode_value()?.ignore()?;
                                            },
                                        }
                                    }else{
                                        entry.decode_value()?.ignore()?;
                                    };
                                    entry.decode_end()?;
                                }
                            },
                            v => v.mismatch("map from field names or indices to field values")?,
                        }
                        #(
                            let #field_idents = #field_idents.ok_or(#schema_error::MissingField{field_name:#field_literals})?;
                        )*
                        ::std::result::Result::Ok(#type_ident {
                            #(
                                #field_idents
                            ),*
                        })
                    }
                }
            }),
            ParsedFields::Unnamed(ParsedFieldsUnnamed {
                field_count,
                field_types,
                field_index_idents,
                field_named_idents,
            }) => Ok(quote! {
                #imp {
                    fn deserialize(decoder: #any_decoder_type<'_,'de,P>, ctx: &mut #context_type) -> #result_type<Self>{
                        match decoder.decode( #decode_hint_type::TupleStruct{name:#type_name, len:#field_count})?{
                            #decoder_view_type::Seq(mut decoder) => {
                                let result=#type_ident(
                                    #(
                                        {
                                            let x = <#field_types as #deserialize_trait<'de, P> >::deserialize(
                                                decoder.decode_next()?
                                                    .ok_or(#schema_error::TupleTooShort)?,
                                                ctx
                                            )?;
                                            x
                                        },
                                    )*
                                );
                                decoder.ignore()?;
                                ::std::result::Result::Ok(result)
                            },
                            v => v.mismatch("seq")?
                        }
                    }
                }
            }),
            ParsedFields::Unit => Ok(quote! {
                #imp {
                    fn deserialize(decoder: #any_decoder_type<'_,'de,P>, ctx: &mut #context_type) -> #result_type<Self>{
                        match decoder.decode( #decode_hint_type::UnitStruct{name:#type_name})?{
                            #decoder_view_type::Primitive(#primitive_type::Unit) => ::std::result::Result::Ok(#type_ident),
                            v => v.mismatch("unit")?,
                        }
                    }
                }
            }),
        },
        Data::Enum(data) => {
            let ParsedEnum {
                variant_idents,
                variant_literals,
                variant_indices,
            } = ParsedEnum::new(data);
            let mut matches: Vec<TokenStream> = vec![];
            for (variant_index, variant) in data.variants.iter().enumerate() {
                let Variant {
                    attrs: _,
                    ident: variant_ident,
                    fields,
                    discriminant: _,
                } = variant;
                match ParsedFields::new(fields) {
                    ParsedFields::Named(
                        ParsedFieldsNamed {
                            field_idents,
                            field_types,
                            field_literals,
                            field_indices
                        }) => {
                        matches.push(quote! {
                            #variant_index => {
                                let hint = #decode_variant_hint_type::StructVariant{
                                    fields: &[
                                        #(
                                            #field_literals
                                        ),*
                                    ],
                                };
                                #(
                                    let mut #field_idents : #option_type<#field_types> = #option_type::None;
                                )*
                                let decoder = decoder.decode_variant(hint)?;
                                match decoder {
                                    #decoder_view_type::Map(mut decoder) => {
                                        while let Some(mut entry) = decoder.decode_next()?{
                                            let field_index:Option<usize> = match entry.decode_key()?.decode(#decode_hint_type::Identifier)?{
                                                #decoder_view_type::String(name) => match &*name{
                                                    #(
                                                        #field_literals => Some(#field_indices),
                                                    )*
                                                    _ => None,
                                                },
                                                #decoder_view_type::Primitive(x) => Some(<usize as TryFrom<#primitive_type>>::try_from(x)?),
                                                v => v.mismatch("field name or index")?,
                                            };
                                            if let Some(field_index)=field_index{
                                                match field_index {
                                                    #(
                                                        #field_indices => {
                                                            let value = entry.decode_value()?;
                                                            #field_idents = Some(<#field_types as #deserialize_trait<'de, P>>::deserialize(value, ctx)?);
                                                        }
                                                    )*
                                                    _ => entry.decode_value()?.ignore()?,
                                                }
                                            }else{
                                                entry.decode_value()?.ignore()?;
                                            }
                                            entry.decode_end()?;
                                        }
                                    },
                                    v => v.mismatch("expected map")?
                                }
                                #(
                                    let #field_idents = #field_idents.ok_or(#schema_error::MissingField{field_name:#field_literals})?;
                                )*
                                #type_ident::#variant_ident {
                                    #(
                                        #field_idents
                                    ),*
                                }
                            },
                        });
                    }

                    ParsedFields::Unnamed(
                        ParsedFieldsUnnamed {
                                              field_count,
                                              field_types,
                            field_index_idents,
                            field_named_idents,
                        })
                    => {
                        matches.push(quote! {
                            #variant_index => {
                                match decoder.decode_variant( #decode_variant_hint_type::TupleVariant{ len: #field_count })?{
                                    #decoder_view_type::Seq(mut decoder) => {
                                        let result=#type_ident::#variant_ident(
                                            #(
                                                {
                                                    let x = <#field_types as #deserialize_trait<'de, P> >::deserialize(
                                                        decoder.decode_next()?
                                                            .ok_or(#schema_error::TupleTooShort)?,
                                                        ctx
                                                    )?;
                                                    x
                                                },
                                            )*
                                        );
                                        decoder.ignore()?;
                                        result
                                    },
                                    v => v.mismatch("seq")?
                                }
                            },
                        })
                    }

                    ParsedFields::Unit => matches.push(quote! {
                        #variant_index => {
                            let variant = decoder.decode_variant(#decode_variant_hint_type::UnitVariant)?;
                            variant.ignore()?;
                            #type_ident::#variant_ident
                        },
                    }),
                }
            }
            Ok(quote! {
                #imp {
                    fn deserialize(decoder: #any_decoder_type<'_,'de,P>, ctx: &mut #context_type) -> #result_type<Self>{
                        let hint = #decode_hint_type::Enum {
                            variants: &[
                                #(
                                    #variant_literals
                                ),*
                            ],
                            name: #type_name,
                        };
                        let decoder = decoder.decode( hint)?;
                        match decoder {
                            #decoder_view_type::Enum(mut decoder) => {
                                let variant_index = {
                                    let disc = decoder.decode_discriminant()?;
                                    let disc = disc.decode( #decode_hint_type::Identifier)?;
                                    match disc {
                                        #decoder_view_type::Primitive(variant_index) => usize::try_from(variant_index)?,
                                        #decoder_view_type::String(disc) => match &*disc {
                                            #(
                                                #variant_literals => #variant_indices,
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
                                decoder.decode_end()?;
                                ::std::result::Result::Ok(result)
                            },
                            v => v.mismatch("enum")?,
                        }
                    }
                }
            })
        }
        Data::Union(u) => {
            return Err(syn::Error::new(
                u.union_token.span,
                "Cannot derive Deserialize for for unions.",
            ))?;
        }
    }
}