use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, LitStr, Variant};

use crate::generics::DeriveGenerics;
use crate::parsed_enum::ParsedEnum;
use crate::parsed_fields::{ParsedFields, ParsedFieldsNamed, ParsedFieldsUnnamed};

pub fn derive_serialize_update_impl(input: &DeriveInput) -> Result<TokenStream, syn::Error> {
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
    } = DeriveGenerics::new(
        generics,
        &quote! {::marshal_update::ser::SerializeUpdate<W>},
    );
    let gen_encoder_trait = quote! { ::marshal::encode::GenEncoder };
    let serialize_update_trait = quote! { ::marshal_update::ser::SerializeUpdate };
    let context_type = quote! { ::marshal::context::Context };
    let type_name = LitStr::new(&format!("{}", type_ident), type_ident.span());
    let any_gen_encoder_type = quote!(::marshal::encode::AnyGenEncoder);

    let anyhow = quote!(::marshal::reexports::anyhow);
    let result_type = quote!(#anyhow::Result);

    let imp = quote!(impl<#(#generic_params,)* W: #gen_encoder_trait> #serialize_update_trait<W> for #type_ident <#(#generic_args),*>);
    match data {
        Data::Struct(data) => match ParsedFields::new(&data.fields) {
            ParsedFields::Unit => Ok(quote! {
                #imp {
                    fn serialize_update<'w, 'en>(&self, stream: &mut Self::Stream, encoder: #any_gen_encoder_type<'w, 'en, W>, mut ctx: #context_type) -> #result_type<()> {
                        encoder.encode_unit_struct(#type_name)
                    }
                }
            }),
            ParsedFields::Named(ParsedFieldsNamed {
                field_idents,
                field_types,
                field_literals,
                field_indices: _,
            }) => Ok(quote! {
                #imp {
                    fn serialize_update<'w,'en>(&self, stream:&mut Self::Stream, encoder: #any_gen_encoder_type<'w,'en, W>, mut ctx: #context_type) -> #result_type<()> {
                        let mut encoder = encoder.encode_struct( #type_name, &[
                                #(
                                    #field_literals
                                ),*
                            ])?;
                        #(
                            <#field_types as #serialize_update_trait<W>>::serialize_update(
                                &self.#field_idents,
                                &mut stream.#field_idents,
                                encoder.encode_field()?,ctx.reborrow()
                            )?;
                        )*
                        encoder.end()?;
                        ::std::result::Result::Ok(())
                    }
                }
            }),
            ParsedFields::Unnamed(ParsedFieldsUnnamed {
                field_count,
                field_types,
                field_index_idents,
                field_named_idents: _,
            }) => Ok(quote! {
                #imp {
                    fn serialize_update<'w,'en>(&self, stream:&mut Self::Stream, encoder: #any_gen_encoder_type<'w,'en, W>, mut ctx: #context_type) -> #result_type<()> {
                        let mut encoder = encoder.encode_tuple_struct( #type_name, #field_count)?;
                        #(
                            <#field_types as #serialize_update_trait<W>>::serialize_update(
                                &self.#field_index_idents,
                                &mut stream.#field_index_idents,
                                encoder.encode_field()?, ctx.reborrow()
                            )?;
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
                                    #serialize_update_trait::<W>::serialize(#field_idents, encoder.encode_field()?, ctx.reborrow())?;
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
                                    #serialize_update_trait::<W>::serialize(#field_named_idents, encoder.encode_field()?, ctx.reborrow())?;
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
                    fn serialize(&self, encoder: #any_gen_encoder_type<'_,W>, mut ctx: #context_type) -> #result_type<()> {
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
