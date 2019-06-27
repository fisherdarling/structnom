#![recursion_limit = "128"]

extern crate proc_macro;

use crate::proc_macro::TokenStream;

use std::convert::TryInto;

use quote::{quote, ToTokens};
use syn::{
    parse2, parse_macro_input, AttrStyle, Attribute, Data, DataEnum, DataStruct, DeriveInput,
    Field, Fields, FieldsNamed, FieldsUnnamed, Generics, Ident, Lit, LitInt, Meta, MetaList,
    MetaNameValue, NestedMeta, Variant, Span,
};

use proc_macro2::TokenTree;

use serde::{Serialize, Deserialize};
use std::fmt;
use std::borrow::Borrow;

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


#[proc_macro_derive(StructNom, attributes(snom))]
pub fn structnom_derive(input: TokenStream) -> TokenStream {
    input
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
        Endian::little()
    }

    pub fn big() -> Self {
        Endian::big()
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




