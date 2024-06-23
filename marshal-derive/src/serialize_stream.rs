use crate::generics::DeriveGenerics;
use crate::ident_to_lit;
use crate::parsed_enum::ParsedEnum;
use crate::parsed_fields::{ParsedFields, ParsedFieldsNamed, ParsedFieldsUnnamed};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Fields, GenericParam, Generics, LitStr, TypeParam,
    Variant,
};

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
    let any_encoder_type = quote!(::marshal::encode::AnyEncoder);

    let anyhow = quote!(::marshal::reexports::anyhow);
    let result_type = quote!(#anyhow::Result);

    let imp = quote!(impl<#(#generic_params,)*> #serialize_stream_trait for #type_ident <#(#generic_args),*>);

    let stream_ident = format_ident!("{}Stream", type_ident);

    match data {
        Data::Struct(data) => match ParsedFields::new(&data.fields) {
            ParsedFields::Unit => Ok(quote! {
                struct #stream_ident;
                #imp {
                    type Stream = #stream_ident;
                    fn start_stream(&self, ctx:&mut #context_type)->#result_type<#stream_ident>{
                        Ok(#stream_ident)
                    }
                }
            }),
            ParsedFields::Named(ParsedFieldsNamed {
                field_idents,
                field_types,
                field_literals,
                field_indices,
            }) => Ok(quote! {
                struct #stream_ident{
                    #(
                        #field_idents: #field_types,
                    )*
                }
                #imp {
                    type Stream = #stream_ident;
                    fn start_stream(&self, ctx:&mut #context_type) -> #result_type<#stream_ident>{
                        Ok(#stream_ident{
                            #(
                                #field_idents: self.#field_idents.start_stream(ctx)?,
                            )*
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
                struct #stream_ident(
                    #(
                        #field_types
                    ),*
                );
                #imp {
                    type Stream = #stream_ident;
                    fn start_stream(&self, ctx:&mut #context_type) -> #result_type<#stream_ident>{
                        Ok(#stream_ident(
                            #(
                                self.#field_index_idents.start_stream(ctx)?
                            ),*
                        ))
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
                        field_types,
                        field_literals,
                        field_indices,
                    }) => {
                        matches.push(quote! {
                            Self::#variant_ident{ #(#field_idents),* } => {
                                todo!();
                            },
                        });
                    }
                    ParsedFields::Unnamed(ParsedFieldsUnnamed {
                        field_count,
                        field_types,
                        field_index_idents,
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
