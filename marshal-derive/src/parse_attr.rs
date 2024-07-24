use syn::meta::ParseNestedMeta;
use syn::spanned::Spanned;
use syn::{Attribute, LitStr, Token};

pub struct ParsedAttrs {
    pub rename: Option<LitStr>,
}
impl ParsedAttrs {
    pub fn new(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut rename = None;
        for attrs in attrs {
            if attrs.path().is_ident("marshal") {
                attrs.parse_nested_meta(|x: ParseNestedMeta| {
                    x.input.parse::<Token![=]>()?;
                    if x.path.is_ident("rename") {
                        if rename.is_some() {
                            return Err(syn::Error::new(x.path.span(), "two renames"));
                        }
                        rename = Some(x.input.parse::<LitStr>()?);
                        return Ok(());
                    } else {
                        return Err(syn::Error::new(x.path.span(), "attribute not recognized"));
                    }
                })?;
            }
        }
        Ok(ParsedAttrs { rename })
    }
}
