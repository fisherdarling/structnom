use quote::quote;
use syn::{Attribute, Meta};

use crate::attribute::*;
// "pre_tag" | "pre_take"

use crate::{int_once, int_slice};
// fn int_slice(list: &MetaList) -> proc_macro2::TokenStream {

pub fn parse_value(attr: &Attribute) -> proc_macro2::TokenStream {
    let kind = get_path_ident(&attr.path).to_string();

    match kind.as_ref() {
        "tag" => {
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
        "take" => {
            let list = match attr.parse_meta().unwrap() {
                Meta::List(l) => l,
                _ => unimplemented!("Expected a list of arguments for `pre_tag`"),
            };

            let int = int_once(&list);

            let expanded = quote! {
                take!(#int)
            };

            expanded
        }
        "parser" => {
            let func = get_path(attr);

            let expanded = quote! {
                call!(#func)
            };

            expanded
        }
        _ => unimplemented!("Unimplemented value attribute"),
    }
}
