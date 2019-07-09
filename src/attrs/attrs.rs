use crate::attrs;

use quote::quote;

use syn::{
    parse2, parse_macro_input, AttrStyle, Attribute, Data, DataEnum, DataStruct, DeriveInput,
    ExprPath, Fields, FieldsNamed, FieldsUnnamed, Generics, Ident, Lit, LitInt,
    Meta::*,
    MetaList, MetaNameValue,
    NestedMeta::{self, *},
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
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Container {
    parser: Option<ExprPath>,
    switch: Option<ExprPath>,
    bits: bool,
    // debug: bool,
    // iterating: bool,
    // streaming: bool,
    // range: (LitInt, LitInt),
}

impl Container {
    pub fn from_input(ast: &DeriveInput) -> Container {
        let mut container = Container::default();

        for nested_meta in ast.attrs.iter().filter_map(get_snom_metas) {
            // println!("{:?}", nested_meta);
            for meta in nested_meta {
                match meta {
                    // #[snom(parser = "path::to::parser")]
                    Meta(NameValue(ref m)) if m.ident == "parser" => {
                        let path = attrs::get_expr_path(&m.lit).unwrap();
                        // println!("parser = \"{}\"", quote!(#path));
                        container.parser.replace(path);
                    }
                    // #[snom(switch = "path::to::switch")]
                    Meta(NameValue(ref m)) if m.ident == "switch" => {
                        let path = attrs::get_expr_path(&m.lit).unwrap();
                        // println!("parser = \"{}\"", quote!(#path));
                        container.switch.replace(path);
                    }
                    // #[snom(parser = "path::to::parser")]
                    Meta(Word(ref ident)) if ident == "bits" => {
                        container.bits = true;
                    }
                    _ => panic!(),
                }
            }
        }
        container
    }
}

/// A field describes the possible attributes on the field
/// of either a struct or enum.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub parser: Option<ExprPath>,
    pub skip: bool,
    pub take: Option<LitInt>,
    pub bits: Option<Bits>,
    pub call: Option<Vec<ExprPath>>,
    // debug: bool
}

impl Field {
    pub fn from_field(ast: &syn::Field) -> Field {
        // println!("Field attributes: {:?}", field.attrs);

        let mut field = Field::default();

        for nested_meta in ast.attrs.iter().filter_map(get_snom_metas) {
            // println!("{:?}", nested_meta);
            for meta in nested_meta {
                match meta {
                    // #[snom(parser = "path::to::parser")]
                    Meta(NameValue(ref m)) if m.ident == "parser" => {
                        let path = attrs::get_expr_path(&m.lit).unwrap();
                        // println!("parser = \"{}\"", quote!(#path));
                        field.parser.replace(path);
                    }
                    // #[snom(skip)]
                    Meta(Word(ref i)) if i == "skip" => {
                        // println!("skip");
                        field.skip = true;
                    }
                    // #[snom(take(4))]
                    Meta(List(ref l)) if l.ident == "take" => {
                        if let Some(Literal(Lit::Int(lit))) = l.nested.iter().next() {
                            // println!("take({})", quote!(#lit));
                            field.take.replace(lit.clone());
                        } else {
                            panic!()
                        }
                    }
                    // #[snom(bits(4))] or
                    // #[snom(bits(4, some_pattern))]
                    Meta(List(ref l)) if l.ident == "bits" => {
                        let mut iter = l.nested.iter();

                        let count = match iter.next() {
                            Some(Literal(Lit::Int(lit))) => lit.clone(),
                            _ => panic!(),
                        };

                        if l.nested.len() == 2 {
                            let pattern = match iter.next() {
                                Some(Literal(Lit::Int(lit))) => lit.clone(),
                                _ => panic!(),
                            };

                            field.bits.replace(Bits::Tag(count, pattern));
                        } else {
                            field.bits.replace(Bits::Take(count));
                        }
                    }
                    // #[snom(call = "path::to::call")]
                    Meta(NameValue(ref m)) if m.ident == "call" => {
                        let path = attrs::get_expr_path(&m.lit).unwrap();
                        // println!("call = \"{}\"", quote!(#path));
                        field.call.get_or_insert(Vec::new()).push(path);
                    }
                    _ => panic!(),
                }
            }
        }

        field
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Variant {
    pub match_value: Option<Lit>,
    pub skip: bool,
    pub take: Option<LitInt>,
    pub call: Option<Vec<ExprPath>>,
    pub range: Option<Range>,
}

impl Variant {
    pub fn from_variant(ast: &syn::Variant) -> Variant {
        let mut variant = Variant::default();

        for nested_meta in ast.attrs.iter().filter_map(get_snom_metas) {
            for meta in nested_meta {
                // println!("Meta: {:?}", meta);
                // println!("Meta ======== ============ \n{:?}\n\n", nested_meta);
                match meta {
                    // #[snom(val = x30)]
                    Meta(NameValue(ref m)) if m.ident == "val" => {
                        variant.match_value.replace(m.lit.clone());
                    }
                    // #[snom(skip)]
                    Meta(Word(ref i)) if i == "skip" => {
                        // println!("skip");
                        variant.skip = true;
                    }
                    // #[snom(take(4))]
                    Meta(List(ref l)) if l.ident == "take" => {
                        if let Some(Literal(Lit::Int(lit))) = l.nested.iter().next() {
                            // println!("take({})", quote!(#lit));
                            variant.take.replace(lit.clone());
                        } else {
                            panic!()
                        }
                    }
                    // #[snom(call = "path::to::call")]
                    Meta(NameValue(ref m)) if m.ident == "call" => {
                        let path = attrs::get_expr_path(&m.lit).unwrap();
                        // println!("call = \"{}\"", quote!(#path));
                        variant.call.get_or_insert(Vec::new()).push(path);
                    }
                    // #[snom(range(start = lit_int))]
                    // #[snom(range(skip = lit_int))]
                    // #[snom(range(end = lit_int))]
                    // #[snom(range(skip))]
                    Meta(List(ref l)) if l.ident == "range" => {
                        let range = match l.nested.iter().next() {
                            Some(Meta(NameValue(ref m))) => {
                                if m.ident == "start" {
                                    let lit = get_lit_int(&m.lit);
                                    Range::Start(lit)
                                } else if m.ident == "skip" {
                                    let lit = get_lit_int(&m.lit);
                                    Range::Skip(Some(lit))
                                } else if m.ident == "end" {
                                    let lit = get_lit_int(&m.lit);
                                    Range::End(lit)
                                } else {
                                    panic!()
                                }
                            }
                            Some(Meta(Word(ident))) if ident == "skip" => {
                                Range::Skip(None)
                            }
                            _ => panic!()
                        };

                        variant.range.replace(range);
                    }
                    _ => panic!(),
                }
            }
        }

        variant
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Bits {
    Tag(LitInt, LitInt),
    Take(LitInt),
}

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

pub fn get_lit_int(lit: &Lit) -> LitInt {
    if let Lit::Int(lit) = lit {
        lit.clone()
    } else {
        panic!()
    }
}