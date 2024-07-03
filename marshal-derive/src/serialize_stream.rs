use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, LitStr, Variant};

use crate::generics::DeriveGenerics;
use crate::parsed_enum::ParsedEnum;
use crate::parsed_fields::{ParsedFields, ParsedFieldsNamed, ParsedFieldsUnnamed};

pub fn derive_serialize_stream_impl(input: &DeriveInput) -> Result<TokenStream, syn::Error> {
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
    } = DeriveGenerics::new(generics, &quote! {::marshal_update::ser::SerializeStream});
    let serialize_stream_trait = quote! { ::marshal_update::ser::SerializeStream };
    let context_type = quote! { ::marshal::context::Context };
    let type_name = LitStr::new(&format!("{}", type_ident), type_ident.span());

    let anyhow = quote!(::marshal::reexports::anyhow);
    let result_type = quote!(#anyhow::Result);

    let imp = quote!(impl<#(#generic_params,)*> #serialize_stream_trait for #type_ident <#(#generic_args),*>);

    let stream_ident = format_ident!("{}Stream", type_ident);

    match data {
        Data::Struct(data) => match ParsedFields::new(&data.fields)? {
            ParsedFields::Unit => Ok(quote! {
                pub struct #stream_ident;
                #imp {
                    type Stream = #stream_ident;
                    fn start_stream(&self, mut ctx: #context_type)->#result_type<#stream_ident>{
                        Ok(#stream_ident)
                    }
                }
            }),
            ParsedFields::Named(ParsedFieldsNamed {
                field_idents,
                field_var_idents:_,
                field_types,
                field_literals: _,
                field_indices: _,
            }) => Ok(quote! {
                pub struct #stream_ident{
                    #(
                        #field_idents: <#field_types as #serialize_stream_trait>::Stream,
                    )*
                }
                #imp {
                    type Stream = #stream_ident;
                    fn start_stream(&self, mut ctx: #context_type) -> #result_type<#stream_ident>{
                        Ok(#stream_ident{
                            #(
                                #field_idents: self.#field_idents.start_stream(ctx.reborrow())?,
                            )*
                        })
                    }
                }
            }),
            ParsedFields::Unnamed(ParsedFieldsUnnamed {
                field_count: _,
                field_types,
                field_index_idents,
                field_named_idents: _,
            }) => Ok(quote! {
                pub struct #stream_ident(
                    #(
                        #field_types
                    ),*
                );
                #imp {
                    type Stream = #stream_ident;
                    fn start_stream(&self, mut ctx: #context_type) -> #result_type<#stream_ident>{
                        Ok(#stream_ident(
                            #(
                                self.#field_index_idents.start_stream(ctx.reborrow())?
                            ),*
                        ))
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
                match ParsedFields::new(&variant.fields)? {
                    ParsedFields::Named(ParsedFieldsNamed {
                        field_idents,
                        field_var_idents:_,
                        field_types: _,
                        field_literals: _,
                        field_indices: _,
                    }) => {
                        matches.push(quote! {
                            Self::#variant_ident{ #(#field_idents),* } => {
                                todo!();
                            },
                        });
                    }
                    ParsedFields::Unnamed(ParsedFieldsUnnamed {
                        field_count: _,
                        field_types: _,
                        field_index_idents: _,
                        field_named_idents,
                    }) => {
                        matches.push(quote! {
                            Self::#variant_ident(#( #field_named_idents ),*) => {
                                todo!();
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
