use crate::ident_to_lit;
use proc_macro2::Ident;
use syn::{DataEnum, LitStr};

pub struct ParsedEnum<'a> {
    pub variant_idents: Vec<&'a Ident>,
    pub variant_literals: Vec<LitStr>,
    pub variant_indices: Vec<usize>,
}

impl<'a> ParsedEnum<'a> {
    pub fn new(enu: &'a DataEnum) -> Self {
        let mut variant_idents = vec![];
        let mut variant_literals = vec![];
        let mut variant_indices = vec![];
        for (index, variant) in enu.variants.iter().enumerate() {
            variant_idents.push(&variant.ident);
            variant_literals.push(ident_to_lit(&variant.ident));
            variant_indices.push(index);
        }
        ParsedEnum {
            variant_idents,
            variant_literals,
            variant_indices,
        }
    }
}
