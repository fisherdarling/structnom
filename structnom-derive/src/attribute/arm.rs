use quote::quote;
use syn::{Attribute, Meta};

use crate::attribute::*;

use crate::{get_int_literals, int_once, int_range, int_slice};
// fn int_slice(list: &MetaList) -> proc_macro2::TokenStream {

// "range" | "byte" | "bytes" => self.parse_arm(attr),
pub fn parse_arm(attr: &Attribute) -> proc_macro2::TokenStream {
    let kind = get_path_ident(&attr.path).to_string();

    let list = match attr.parse_meta().unwrap() {
        Meta::List(l) => l,
        _ => unimplemented!("Expected a list of arguments for `match` tags"),
    };

    match kind.as_ref() {
        "byte" => int_once(&list),
        "bytes" => int_slice(&list),
        "range" => {
            let lits = get_int_literals(&list);

            let start = &lits[0];
            let end = &lits[1];

            let expanded = quote! {
                #start ..= #end
            };

            // println!("{}", expanded);

            expanded
        }
        _ => unimplemented!("Unimplemented match arm"),
    }
}
