use crate::attrs;

use syn::{
    parse2, parse_macro_input, AttrStyle, Attribute, Data, DataEnum, DataStruct, DeriveInput,
    ExprPath, Fields, FieldsNamed, FieldsUnnamed, Generics, Ident, Lit, LitInt, Meta::*, MetaList,
    MetaNameValue, NestedMeta::*,
};

/// A container describes StructNom parsing information
/// at the level of an entire struct or enum.
///
/// Available options for enums and structs are:
///
/// ```
/// #[snom(parser = "path::to::parser")]
/// #[snom(iterating)]
/// #[snom(streaming)]
/// #[snom(debug)]
/// #[snom(bits)]
/// ```
/// For enums either a switch, parser, or a range with a switch
/// *must* be defined:
///
/// ```
/// #[snom(switch = "path::to::switch")]
/// #[snom(range("path::to::switch", start_lit, stop_lit))]
/// ```
///
/// The switch option describes what will be matched on for a large
/// switch statement. For example, switching on `nom::number::complete::le_u8`
/// will match on a little endian byte to determine the parser to use
/// for an Enum variant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    parser: Option<ExprPath>,
    switch: Option<ExprPath>,
    bits: bool,
    // debug: bool,
    // iterating: bool,
    // streaming: bool,
    // range: (LitInt, LitInt),
}

/// A field describes the possible attributes on the field
/// of either a struct or enum.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Field {
    parser: Option<ExprPath>,
    skip: bool,
    take: Option<Take>,
    bits: Option<Bits>,
    call: Option<Lit>,
    // debug: bool
}

impl Field {
    pub fn from_field(field: &syn::Field) -> Field {
        println!("Field attributes: {:?}", field.attrs);

        for nested_meta in field.attrs.iter().filter_map(get_snom_metas) {
            println!("{:?}", nested_meta);
            for meta in nested_meta {
                match meta {
                    Meta(NameValue(ref m)) if m.ident == "parser" => {
                        let path = attrs::get_expr_path(&m.lit).unwrap();
                        println!("{:?}", path);
                    }
                    _ => panic!(),
                }
            }
        }

        unimplemented!()
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Variant {
    match_value: Option<Lit>,
    skip: bool,
    take: Option<Take>,
    call: Option<Lit>,
    range: Option<Range>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bits {
    Tag { count: Lit, value: Lit },
    Take(Take),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Take(LitInt);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Range {
    Start(LitInt),
    Skip(Option<LitInt>),
    End(LitInt),
}

pub fn get_snom_metas(attr: &syn::Attribute) -> Option<Vec<syn::NestedMeta>> {
    if attr.path.segments.len() == 1 && attr.path.segments[0].ident == "snom" {
        match attr.interpret_meta() {
            Some(List(ref meta)) => Some(meta.nested.iter().cloned().collect()),
            _ => None,
        }
    } else {
        None
    }
}
