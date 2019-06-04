pub mod arm;
pub mod post;
pub mod pre;
pub mod value;

use proc_macro2::TokenStream as Tokens;

use quote::quote;
use syn::{Attribute, Ident, Lit, Meta, MetaNameValue};

#[derive(Debug, Default)]
pub struct ParserList {
    pub(crate) pre: Vec<Tokens>,
    pub(crate) match_arm: Option<Tokens>,
    pub(crate) value: Option<Tokens>,
    pub(crate) post: Vec<Tokens>,
}

impl ParserList {
    pub fn from_attributes(attrs: &[Attribute]) -> Self {
        let mut list = ParserList::default();

        list.parse_attributes(attrs);

        list
    }

    fn parse_arm(&mut self, attr: &Attribute) {
        let parser = arm::parse_arm(attr);

        self.match_arm.replace(parser);
    }

    fn parse_pre(&mut self, attr: &Attribute) {
        let parser = pre::parse_pre(attr);

        self.pre.push(parser);
    }

    fn parse_post(&mut self, attr: &Attribute) {
        let parser = post::parse_post(attr);

        self.post.push(parser);
    }

    fn parse_value(&mut self, attr: &Attribute) {
        let parser = value::parse_value(attr);

        self.value.replace(parser);
    }

    pub fn parse_attributes(&mut self, attrs: &[Attribute]) {
        for attr in attrs {
            let kind = get_path_ident(&attr.path).to_string();

            // TODO: Add pre, post, value, match path names.

            match kind.as_ref() {
                "call" | "pre_tag" | "pre_take" => self.parse_pre(attr),
                "range" | "byte" | "bytes" => self.parse_arm(attr),
                "parser" | "tag" | "take" => self.parse_value(attr),
                "debug" => self.parse_post(attr),
                _ => unimplemented!("Unimplemented StructNom parser"),
            }
        }
    }
}

pub fn get_path_ident(path: &syn::Path) -> Ident {
    path.segments.iter().next().unwrap().ident.clone()
}

pub fn get_path(attr: &Attribute) -> syn::Path {
    match attr.parse_meta().expect("Unable to parse attribute") {
        Meta::NameValue(MetaNameValue {
            lit: Lit::Str(lit_str),
            ..
        }) => lit_str
            .parse()
            .expect("Unable to create path from attribute value."),
        _ => {
            panic!("Expected a str literal");
        }
    }
}
