use quote::quote;
use syn::{Attribute, Meta};

use crate::attribute::*;

pub fn parse_post(attr: &Attribute) -> proc_macro2::TokenStream {
    let kind = get_path_ident(&attr.path).to_string();

    let word = match attr.parse_meta().unwrap() {
        Meta::Word(ident) => ident.to_string(),
        _ => unimplemented!("Expected a list of arguments for `match` tags"),
    };

    match word.as_ref() {
        "debug" => {
            let expanded = quote! {
                value!(println!("{:?}", value))
            };

            expanded
        }
        _ => unimplemented!("Unimplemented post parser"),
    }
}
