use proc_macro2::TokenStream;
use quote::quote;
// use syn::parse::{Lookahead1, Parse, ParseBuffer, ParseStream, Peek};
// use syn::{parenthesized, punctuated::Punctuated, Attribute, LitInt, LitStr, Meta, Token, token::Token};

use syn::{
    parse2, parse_macro_input, AttrStyle, Attribute, Data, DataEnum, DataStruct, DeriveInput,
    Field, Fields, FieldsNamed, FieldsUnnamed, Generics, Ident, Lit, LitInt, Meta, MetaList,
    MetaNameValue, NestedMeta, Variant,
};

use crate::attr::SnomArg;

// let expanded = gen_struct_impl(name, &input.attrs, generics, data);

pub struct EnumGenerator {
    name: Ident,
    args: Vec<SnomArg>,
    generics: Generics,
    data: DataEnum,
    // state: EnumState
}

impl EnumGenerator {
    pub fn new(
        name: Ident,
        attrs: Vec<Attribute>,
        generics: Generics,
        data: DataEnum,
    ) -> EnumGenerator {
        let args: Vec<_> = attrs
            .into_iter()
            .map(SnomArg::parse)
            .flatten()
            .flatten()
            .collect();

        EnumGenerator {
            name,
            args,
            generics,
            data,
        }
    }
}
