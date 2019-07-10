#![recursion_limit = "128"]

extern crate proc_macro;

use crate::proc_macro::TokenStream;

use std::convert::TryInto;

use quote::{quote, ToTokens};
use syn::{
    parse2, parse_macro_input, AttrStyle, Attribute, Data, DataEnum, DataStruct, DeriveInput,
    Field, Fields, FieldsNamed, FieldsUnnamed, Generics, Ident, Lit, LitInt, Meta, MetaList,
    MetaNameValue, NestedMeta, Variant,
};

use proc_macro2::Span;

use proc_macro2::TokenTree;

use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::fmt;

mod attrs;
mod fields;

/// generate_structnom!(r#"
///     endian = native, little, big
///     streaming = true, false, both
///     iterating = true, false, both    
///     verbose-errors = true, false
///     vector-style = {
///         length = endian_u8, fn -> Integer
///         terminal = None, &[u8]
///         included = false, true
///     }
///     string-style = {
///         IsVector,
///         length = endian_u8, fn -> Integer
///         terminal = None, &[u8]
///         included = false, true
///     }
///
///     // primitive numeric types are in the form:
///     // {type}-parser = fn -> Integer
///     // They default to endian_{type}
///     
/// "#);
struct Unit;

mod trait_gen;

// TraitConfig {
//     endian: { Native, Little, Big },
//     streaming: { true, false, both },
//     iterating: { true, false, both },
//     verbose-errors = { true, false },
//     vector-style = VectorStyle {
//         length = Option<"path">,
//         terminal = Option<&[u8]>,
//         included = { false, true },
//     },
//     string-style = StringStyle {
//         IsVector,
//         Style {
//             length = Option<"path">,
//             terminal = Option<&[u8]>,
//             included = false, true
//         }
//     }
// }

#[proc_macro_derive(StructNom, attributes(snom))]
pub fn structnom_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let container = attrs::Container::from_input(&input);

    println!("Container: {:?}", container);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    match input.data {
        syn::Data::Struct(DataStruct {
            fields: Fields::Named(ref f),
            ..
        }) => gen_fields_named(&f),
        syn::Data::Struct(DataStruct {
            fields: Fields::Unnamed(ref f),
            ..
        }) => gen_fields_unnamed(&f),
        syn::Data::Enum(DataEnum { variants, .. }) => gen_variants(variants.iter().collect()),
        _ => panic!("Only works on named struct fields"),
    };

    let expanded = quote! {};

    expanded.into()
}

fn gen_fields_named(fields: &FieldsNamed) {
    for field in &fields.named {
        let config = attrs::Field::from_field(field);
        println!("Config: {:?}", config);
    }
}

fn gen_fields_unnamed(fields: &FieldsUnnamed) {
    for field in &fields.unnamed {
        let config = attrs::Field::from_field(field);
        println!("Config: {:?}", config);
    }
}

fn gen_variants(variants: Vec<&Variant>) {
    for variant in variants {
        let config = attrs::Variant::from_variant(variant);
        println!("Config: {:?}", config);
    }
}

pub(crate) trait Generate {
    fn generate(&mut self) -> proc_macro2::TokenStream;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum Endian {
    Little,
    Big,
}

impl Default for Endian {
    #[cfg(target_endian = "little")]
    fn default() -> Self {
        Endian::Little
    }

    #[cfg(target_endian = "big")]
    fn default() -> Self {
        Endian::Big
    }
}

impl fmt::Display for Endian {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Endian::Big => write!(f, "be"),
            Endian::Little => write!(f, "le"),
        }
    }
}

impl Endian {
    pub fn native() -> Self {
        Endian::default()
    }

    pub fn little() -> Self {
        Endian::Little
    }

    pub fn big() -> Self {
        Endian::Big
    }

    pub fn prefix<S: Borrow<str>>(suffix: S, span: Option<Span>) -> Ident {
        let span = if let Some(span) = span {
            span
        } else {
            Span::call_site()
        };
        unimplemented!()
    }
}
