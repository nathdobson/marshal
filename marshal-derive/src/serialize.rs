use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, LitStr, Variant};

use crate::generics::DeriveGenerics;
use crate::parsed_enum::ParsedEnum;
use crate::parsed_fields::{ParsedFields, ParsedFieldsNamed, ParsedFieldsUnnamed};

pub fn derive_serialize_impl(input: &DeriveInput) -> Result<TokenStream, syn::Error> {
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
    } = DeriveGenerics::new(generics, &quote! {::marshal::ser::Serialize<W>});
    let gen_encoder_trait = quote! { ::marshal::encode::Encoder };
    let serialize_trait = quote! { ::marshal::ser::Serialize };
    let context_type = quote! { ::marshal::context::Context };
    let type_name = LitStr::new(&format!("{}", type_ident), type_ident.span());
    let any_gen_encoder_type = quote!(::marshal::encode::AnyEncoder);

    let anyhow = quote!(::marshal::reexports::anyhow);
    let result_type = quote!(#anyhow::Result);

    let imp = quote!(impl<#(#generic_params,)* W: #gen_encoder_trait> #serialize_trait<W> for #type_ident <#(#generic_args),*>);

    match data {
        Data::Struct(data) => match ParsedFields::new(&data.fields) {
            ParsedFields::Unit => Ok(quote! {
                #imp {
                    fn serialize<'w,'en>(&self, encoder: #any_gen_encoder_type<'w,'en, W>, mut ctx: #context_type) -> #result_type<()> {
                        encoder.encode_unit_struct(#type_name)
                    }
                }
            }),
            ParsedFields::Named(ParsedFieldsNamed {
                field_idents,
                field_types: _,
                field_literals,
                field_indices: _,
            }) => Ok(quote! {
                #imp {
                    fn serialize<'w,'en>(&self, encoder: #any_gen_encoder_type<'w,'en, W>, mut ctx: #context_type) -> #result_type<()> {
                        let mut encoder = encoder.encode_struct( #type_name, &[
                                #(
                                    #field_literals
                                ),*
                            ])?;
                        #(
                            #serialize_trait::<W>::serialize(&self.#field_idents, encoder.encode_field()?, ctx.reborrow())?;
                        )*
                        encoder.end()?;
                        ::std::result::Result::Ok(())
                    }
                }
            }),
            ParsedFields::Unnamed(ParsedFieldsUnnamed {
                field_count,
                field_types: _,
                field_index_idents,
                field_named_idents: _,
            }) => Ok(quote! {
                #imp {
                    fn serialize<'w,'en>(&self, encoder: #any_gen_encoder_type<'w,'en, W>, mut ctx: #context_type) -> #result_type<()> {
                        let mut encoder=encoder.encode_tuple_struct( #type_name, #field_count)?;
                        #(
                            #serialize_trait::<W>::serialize(&self.#field_index_idents, encoder.encode_field()?, ctx.reborrow())?;
                        )*
                        encoder.end()?;
                        #result_type::Ok(())
                    }
                }
            }),
        },
        Data::Enum(data) => {
            let ParsedEnum {
                variant_literals,
                variant_indices: _,
            } = ParsedEnum::new(data);
            let mut matches = vec![];
            for (variant_index, variant) in data.variants.iter().enumerate() {
                let Variant {
                    attrs: _,
                    ident: variant_ident,
                    fields: _,
                    discriminant: _,
                } = variant;
                match ParsedFields::new(&variant.fields) {
                    ParsedFields::Named(ParsedFieldsNamed {
                        field_idents,
                        field_types: _,
                        field_literals,
                        field_indices: _,
                    }) => {
                        matches.push(quote! {
                            Self::#variant_ident{ #(#field_idents),* } => {
                                let mut encoder = encoder.encode_struct_variant( #type_name, &[#( #variant_literals ),*], #variant_index, &[#(#field_literals),*])?;
                                #(
                                    #serialize_trait::<W>::serialize(#field_idents, encoder.encode_field()?, ctx.reborrow())?;
                                )*
                                encoder.end()?;
                                ::std::result::Result::Ok(())
                            },
                        });
                    }
                    ParsedFields::Unnamed(ParsedFieldsUnnamed {
                        field_count,
                        field_types: _,
                        field_index_idents: _,
                        field_named_idents,
                    }) => {
                        matches.push(quote! {
                            Self::#variant_ident(#( #field_named_idents ),*) => {
                                let mut encoder = encoder.encode_tuple_variant( #type_name, &[#( #variant_literals ),*], #variant_index, #field_count)?;
                                #(
                                    #serialize_trait::<W>::serialize(#field_named_idents, encoder.encode_field()?, ctx.reborrow())?;
                                )*
                                encoder.end()?;
                                ::std::result::Result::Ok(())
                            },
                        });
                    }
                    ParsedFields::Unit => {
                        matches.push(quote! {
                            Self::#variant_ident => {
                                encoder.encode_unit_variant( #type_name, &[#( #variant_literals ),*], #variant_index)?;
                                ::std::result::Result::Ok(())
                            },
                        });
                    }
                }
            }
            Ok(quote! {
                #imp {
                    fn serialize<'w,'en>(&self, encoder: #any_gen_encoder_type<'w,'en,W>, mut ctx: #context_type) -> #result_type<()> {
                        match self{
                            #(
                                #matches
                            )*
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
