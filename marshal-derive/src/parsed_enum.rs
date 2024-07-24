use syn::{DataEnum, LitStr};

use crate::ident_to_lit;
use crate::parse_attr::ParsedAttrs;

pub struct ParsedEnum {
    pub variant_literals: Vec<LitStr>,
    pub variant_indices: Vec<usize>,
}

impl ParsedEnum {
    pub fn new(enu: &DataEnum) -> syn::Result<Self> {
        let mut variant_idents = vec![];
        let mut variant_literals = vec![];
        let mut variant_indices = vec![];
        for (index, variant) in enu.variants.iter().enumerate() {
            variant_idents.push(&variant.ident);
            let attrs = ParsedAttrs::new(&variant.attrs)?;
            variant_literals.push(attrs.rename.unwrap_or_else(|| ident_to_lit(&variant.ident)));
            variant_indices.push(index);
        }
        Ok(ParsedEnum {
            variant_literals,
            variant_indices,
        })
    }
}
