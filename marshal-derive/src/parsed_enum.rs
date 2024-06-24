use syn::{DataEnum, LitStr};

use crate::ident_to_lit;

pub struct ParsedEnum {
    pub variant_literals: Vec<LitStr>,
    pub variant_indices: Vec<usize>,
}

impl ParsedEnum {
    pub fn new(enu: &DataEnum) -> Self {
        let mut variant_idents = vec![];
        let mut variant_literals = vec![];
        let mut variant_indices = vec![];
        for (index, variant) in enu.variants.iter().enumerate() {
            variant_idents.push(&variant.ident);
            variant_literals.push(ident_to_lit(&variant.ident));
            variant_indices.push(index);
        }
        ParsedEnum {
            variant_literals,
            variant_indices,
        }
    }
}
