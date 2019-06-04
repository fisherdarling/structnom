use syn::Attribute;
use quote::quote;

use crate::attribute::*;

// get_path_ident

pub fn parse_pre(attr: &Attribute) -> proc_macro2::TokenStream {
    let kind = get_path_ident(&attr.path).to_string();

    match kind.as_ref() {
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