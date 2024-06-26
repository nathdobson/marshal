use proc_macro2::Ident;
use quote::format_ident;
use syn::{Fields, LitStr, Type};
use syn::spanned::Spanned;

pub struct ParsedFieldsNamed<'a> {
    pub field_idents: Vec<&'a Ident>, //
    pub field_types: Vec<&'a Type>,
    pub field_literals: Vec<LitStr>,
    pub field_indices: Vec<usize>,
}

pub struct ParsedFieldsUnnamed<'a> {
    pub field_count: usize,
    pub field_types: Vec<&'a Type>,
    pub field_index_idents: Vec<syn::Index>,
    pub field_named_idents: Vec<Ident>,
}

pub enum ParsedFields<'a> {
    Named(ParsedFieldsNamed<'a>),
    Unnamed(ParsedFieldsUnnamed<'a>),
    Unit,
}

impl<'a> ParsedFields<'a> {
    pub fn new(fields: &'a Fields) -> Self {
        match &fields {
            Fields::Named(fields) => {
                let mut field_idents = vec![];
                let mut field_types = vec![];
                let mut field_literals = vec![];
                let mut field_indices = vec![];
                for (index, field) in fields.named.iter().enumerate() {
                    let ident = field.ident.as_ref().unwrap();
                    field_idents.push(ident);
                    field_types.push(&field.ty);
                    field_literals.push(LitStr::new(&format!("{}", ident), ident.span()));
                    field_indices.push(index);
                }

                ParsedFields::Named(ParsedFieldsNamed {
                    field_idents,
                    field_types,
                    field_literals,
                    field_indices,
                })
            }
            Fields::Unnamed(fields) => {
                let field_count = fields.unnamed.len();
                let mut field_types = vec![];
                let mut field_index_idents = vec![];
                let mut field_named_idents = vec![];
                for (index, field) in fields.unnamed.iter().enumerate() {
                    field_index_idents.push(syn::Index::from(index));
                    field_types.push(&field.ty);
                    field_named_idents.push(format_ident!("_{}", index, span = field.span()));
                }
                ParsedFields::Unnamed(ParsedFieldsUnnamed {
                    field_count,
                    field_types,
                    field_index_idents,
                    field_named_idents,
                })
            }
            Fields::Unit => ParsedFields::Unit,
        }
    }
}
