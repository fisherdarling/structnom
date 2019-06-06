use quote::quote;
use syn::Attribute;

use crate::attribute::*;

use crate::{get_int_literals, int_once, int_range, int_slice};

pub fn parse_pre(attr: &Attribute) -> proc_macro2::TokenStream {
    let kind = get_path_ident(&attr.path).to_string();

    match kind.as_ref() {
        "pre_tag" => {
            let list = match attr.parse_meta().unwrap() {
                Meta::List(l) => l,
                _ => unimplemented!("Expected a list of arguments for `pre_tag`"),
            };

            let slice = int_slice(&list);

            let expanded = quote! {
                tag!(#slice)
            };

            expanded
        }
        "pre_take" => {
            let list = match attr.parse_meta().unwrap() {
                Meta::List(l) => l,
                _ => unimplemented!("Expected a list of arguments for `pre_take`"),
            };

            let int = int_once(&list);

            let expanded = quote! {
                take!(#int)
            };

            expanded
        }
        "call" => {
            let func_path = get_path(attr);

            let expanded = quote! {
                call!(#func_path)
            };

            expanded
        }
        _ => unimplemented!("Unimplemented pre-parser attribute"),
    }
}
