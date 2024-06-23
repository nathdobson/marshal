use proc_macro2::TokenStream;
use quote::quote;
use syn::{GenericParam, Generics, TypeParam};

pub struct DeriveGenerics {
    pub generic_params: Vec<TokenStream>,
    pub generic_args: Vec<TokenStream>,
}

impl DeriveGenerics {
    pub fn new(generics: &Generics, extra_bound: &TokenStream) -> Self {
        let mut generic_params = vec![];
        let mut generic_args = vec![];
        for generic in &generics.params {
            match generic {
                GenericParam::Lifetime(x) => {
                    let p = &x.lifetime;
                    generic_params.push(quote! {#x});
                    generic_args.push(quote!(#p));
                }
                GenericParam::Type(TypeParam {
                    attrs,
                    ident,
                    colon_token,
                    bounds,
                    eq_token,
                    default,
                }) => {
                    let colon_token = colon_token.map_or_else(|| quote!(:), |x| quote!(#x));
                    let bounds = if bounds.is_empty() {
                        quote! {#extra_bound}
                    } else {
                        quote! {#bounds + #extra_bound}
                    };
                    generic_params
                        .push(quote! {#(#attrs)* #ident #colon_token #bounds #eq_token #default});
                    generic_args.push(quote!(#ident));
                }
                GenericParam::Const(x) => {
                    let p = &x.ident;
                    generic_params.push(quote! {#x});
                    generic_args.push(quote!(#p));
                }
            }
        }
        DeriveGenerics {
            generic_params,
            generic_args,
        }
    }
}
