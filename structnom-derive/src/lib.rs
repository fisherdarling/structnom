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

use proc_macro2::TokenTree;

mod gen_trait;

mod attr;
mod attribute;
mod gen;

use attribute::*;

use gen_trait::*;

#[proc_macro_derive(StructNom)]
pub fn nom_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    // let attrs = &input.attrs;
    let generics = input.generics;

    match input.data {
        Data::Struct(data) => {
            let expanded = gen_struct_impl(name, &input.attrs, generics, data);

            TokenStream::from(expanded)
        }
        Data::Enum(data) => {
            let expanded = gen_enum_impl(name, &input.attrs, generics, data);

            TokenStream::from(expanded)
        }
        Data::Union(_) => panic!("Union types are not supported yet."),
    }
}

fn gen_struct_impl(
    name: &Ident,
    attrs: &[Attribute],
    generics: Generics,
    data: DataStruct,
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();;

    let parser = match attrs.get(0) {
        Some(attr) => {
            let meta = attr.parse_meta().unwrap();

            let name_value = match meta {
                // Meta::List(list) => list,
                Meta::NameValue(nv) => nv,
                _ => unimplemented!("Enum impl only supports MetaList"),
            };

            let kind = name_value.ident.to_string();
            let expanded = match kind.as_ref() {
                "parser" => {
                    let func_path = get_path(&attr);

                    quote! {
                        call!(#func_path)
                    }
                }
                _ => unimplemented!("Unimplemented Attribute"),
            };

            // log::debug!("parser: {}", expanded);

            expanded
        }
        None => {
            let fields = data.fields;

            let fields_do_parse = match fields {
                Fields::Named(ref fields) => gen_named_fields(name, fields),
                Fields::Unnamed(ref fields) => gen_unnamed_fields(name, fields),
                Fields::Unit => quote! { value!(Self) },
                // _ => panic!("Fields Unnamed"),
            };

            let expanded = quote! {
                #fields_do_parse
            };

            expanded
        }
    };

    let expanded = quote! {
        impl #impl_generics crate::StructNom for #name #ty_generics #where_clause {
            fn nom(input: &[u8]) -> nom::IResult<&[u8], Self> {
                let res = do_parse!(input,
                            value: #parser >>
                            (value));

                res
            }
        }
    };

    // log::debug!("Gen Struct Impl {}", expanded);

    expanded
}

fn gen_match_arms(parsed_attrs: &[ParserList]) -> Vec<proc_macro2::TokenStream> {
    let mut state = None;
    let mut skip = false;

    let mut ret = Vec::new();

    for parsed in parsed_attrs {
        match parsed.range.clone() {
            Some(s @ MatchRange::Start(_)) => state.replace(s),
            Some(MatchRange::End(lit)) => {
                match state {
                    Some(MatchRange::Start(ref should_be)) => {
                        assert_eq!(should_be.value(), lit.value());
                        log::debug!("Equal Assert");
                    }
                    _ => panic!("range_end must be preceded by range_start"),
                };

                state.replace(MatchRange::End(lit))
            }
            Some(MatchRange::Skip) => {
                skip = true;
                None
            }
            None => None,
        };

        let mut arm;

        state = match state {
            Some(MatchRange::Start(lit)) => {
                let lit = if skip {
                    skip = false;
                    LitInt::new(lit.value() + 1, lit.suffix(), lit.span())
                } else {
                    lit
                };

                arm = quote! {
                    #lit
                };

                let new_lit = LitInt::new(lit.value() + 1, lit.suffix(), lit.span());

                Some(MatchRange::Start(new_lit))
            }
            Some(MatchRange::End(lit)) => {
                arm = quote! {
                    #lit
                };

                None
            }
            None => {
                arm = parsed.match_arm.clone().unwrap();

                None
            }
            _ => unreachable!("State should never be skip."),
        };

        log::debug!("Generated Arm: {}", arm);

        ret.push(arm);
    }

    ret
}

fn gen_enum_impl(
    name: &Ident,
    attrs: &[Attribute],
    generics: Generics,
    data: DataEnum,
) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let meta: Meta = attrs[0]
        .parse_meta()
        .expect("Unable to parse attribute metadata");

    let list = match meta {
        Meta::List(list) => list,
        _ => unimplemented!("Enum impl only supports MetaList"),
    };

    let kind = list.ident.to_string();

    // log::debug!("Enum Attr Kind: {}", kind);

    let switch_parser = get_func_ident(&list);

    // log::debug!("Switch Parser: {}", switch_parser.to_string());

    // let match_arms: Vec<_> = data.variants.iter().map(get_match_arm).collect();

    let attr_parsers: Vec<_> = data
        .variants
        .iter()
        .map(|v| v.attrs.as_slice())
        .map(ParserList::from_attributes)
        .collect();

    let match_arms = gen_match_arms(&attr_parsers);

    let variant_parsers: Vec<_> = data
        .variants
        .iter()
        .zip(attr_parsers.iter())
        .map(|(variant, parsed)| {
            if let Some(ref parser) = parsed.value {
                parser.clone()
            } else {
                match variant.fields {
                    Fields::Named(ref named) => gen_enum_named_fields(name, &variant, named),
                    Fields::Unnamed(ref unnamed) => {
                        gen_enum_unnamed_fields(name, &variant, unnamed)
                    }
                    Fields::Unit => {
                        // log::debug!("Unit Variant Field");
                        let var_ident = &variant.ident;

                        quote! {
                            value!(#name::#var_ident)
                        }
                    }
                }
            }
        })
        .collect();

    let pre_parsers: Vec<proc_macro2::TokenStream> = attr_parsers
        .iter()
        .map(|p| {
            let tokens = &p.pre;
            quote! {
                #(#tokens >>)*
            }
        })
        .collect();

    let post_parsers: Vec<proc_macro2::TokenStream> = attr_parsers
        .iter()
        .map(|p| {
            let tokens = &p.post;
            quote! {
                #(#tokens >>)*
            }
        })
        .collect();

    let expanded = quote! {
        impl #impl_generics crate::StructNom for #name #ty_generics #where_clause {
            fn nom(input: &[u8]) -> nom::IResult<&[u8], Self> {
                do_parse!(input,
                val: switch!(#switch_parser,
                        #(#match_arms => do_parse!(
                            #(#pre_parsers)*
                            value: #variant_parsers >>
                            #(#post_parsers)*
                            (value)
                        ))|*
                    ) >>
                    (val))
            }
        }
    };

    log::debug!("Generated Enum: {}", expanded);

    expanded
}

fn gen_named_fields(name: &Ident, fields: &FieldsNamed) -> proc_macro2::TokenStream {
    let mut attrs = Vec::new();

    let idents: Vec<_> = fields
        .named
        .iter()
        .map(|f| {
            attrs.push(ParserList::from_attributes(&f.attrs));
            f.ident.clone().unwrap()
        })
        .collect();
    let idents2 = idents.clone();

    // let types: Vec<_> = fields.named.iter().map(|f| f.ty.clone()).collect();
    // let parsers: Vec<_> = fields.named.iter().map(gen_field_parser).collect();
    let parsers: Vec<_> = fields
        .named
        .iter()
        .zip(attrs.into_iter())
        .map(|(f, a)| named_field(f, a))
        .collect();

    let expanded = quote! {
        do_parse!(
            #(
                #idents: #parsers >>
            )*
            (#name { #(#idents2),* } )
        )
    };

    expanded
}

fn named_field(field: &Field, mut attrs: ParserList) -> proc_macro2::TokenStream {
    let pre = attrs.pre;
    let post = attrs.post;

    let ty = &field.ty;

    let value = attrs.value.get_or_insert(quote! { call!(<#ty>::nom) });
    let expanded = quote! {
        do_parse!(
            #(#pre >>)*
            value: #value >>
            #(#post >>)*
            (value)
        )
    };

    log::debug!("Named Field: {}", expanded);

    expanded
}

fn gen_enum_named_fields(
    name: &Ident,
    variant: &Variant,
    fields: &FieldsNamed,
) -> proc_macro2::TokenStream {
    let variant_ident = &variant.ident;

    let idents: Vec<_> = fields
        .named
        .iter()
        .map(|f| f.ident.clone().unwrap())
        .collect();
    let idents2 = idents.clone();

    // let types: Vec<_> = fields.named.iter().map(|f| f.ty.clone()).collect();
    let parsers: Vec<_> = fields.named.iter().map(gen_field_parser).collect();

    let expanded = quote! {
        do_parse!(
            #(
                #idents: #parsers >>
            )*
            (#name::#variant_ident { #(#idents2),* } )
        )
    };

    expanded
}

fn gen_enum_unnamed_fields(
    name: &Ident,
    variant: &Variant,
    fields: &FieldsUnnamed,
) -> proc_macro2::TokenStream {
    let variant_ident = &variant.ident;

    let idents: Vec<_> = fields
        .unnamed
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let name = format!("field_{}", i);
            let ident = Ident::new(&name, proc_macro2::Span::call_site());
            ident
        })
        .collect();
    let idents2 = idents.clone();

    let parsers: Vec<_> = fields.unnamed.iter().map(gen_field_parser).collect();

    // log::debug!("Parsers: {:#?}", parsers);

    let expanded = quote! {
        do_parse!(
            #(
                #idents: #parsers >>
            )*
            (#name::#variant_ident ( #(#idents2),* ) )
        )
    };

    // log::debug!("Expanded: {}", expanded);

    expanded
}

fn gen_unnamed_fields(name: &Ident, fields: &FieldsUnnamed) -> proc_macro2::TokenStream {
    let idents: Vec<_> = fields
        .unnamed
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let name = format!("field_{}", i);
            let ident = Ident::new(&name, proc_macro2::Span::call_site());
            ident
        })
        .collect();
    let idents2 = idents.clone();

    let parsers: Vec<_> = fields.unnamed.iter().map(gen_field_parser).collect();

    // log::debug!("Parsers: {:#?}", parsers);

    let expanded = quote! {
        do_parse!(
            #(
                #idents: #parsers >>
            )*
            (#name ( #(#idents2),* ) )
        )
    };

    // log::debug!("Expanded: {}", expanded);

    expanded
}

fn gen_field_parser(field: &Field) -> proc_macro2::TokenStream {
    if field.attrs.is_empty() {
        let ty = &field.ty;

        let expanded = quote! {
            call!(<#ty>::nom)
        };

        expanded
    } else {
        let attr = &field.attrs[0];
        let meta = attr.parse_meta().unwrap();

        let parser = match meta {
            Meta::List(list) => gen_parser_meta_list(&list, field),
            Meta::NameValue(nv) => {
                if attr.path.is_ident("parser") {
                    let func_path = get_path(&attr);

                    quote! {
                        call!(#func_path)
                    }
                } else {
                    panic!("Unknown identifier for name value")
                }
            }
            m => unimplemented!("Meta"),
        };

        parser
    }
}

fn gen_parser_meta_list(list: &MetaList, field: &Field) -> proc_macro2::TokenStream {
    let kind: String = list.ident.to_string();

    match kind.as_str() {
        "call" => {
            let func = get_func_ident(&list);
            let ty = &field.ty;

            let expanded = quote! {
                do_parse!(
                    call!(#func) >>
                    value: call!(<#ty>::nom) >>
                    (value)
                )
            };

            expanded
        }
        "tag" => {
            let slice = int_slice(&list);
            let ty = &field.ty;

            let expanded = quote! {
                do_parse!(
                    tag!(#slice) >>
                    value: call!(<#ty>::nom) >>
                    (value)
                )
            };

            expanded
        }
        // "parser" => {
        //     let func = get_func_ident(&list);

        //     let expanded = quote! {
        //         call!(#func)
        //     };

        //     expanded
        // }
        _ => unimplemented!("Unimplemented field attribute"),
    }
}

fn int_range(list: &MetaList) -> proc_macro2::TokenStream {
    let expanded = quote! { #list };
    // log::debug!("{}", expanded);

    panic!()
}

fn int_slice(list: &MetaList) -> proc_macro2::TokenStream {
    let ints = get_int_literals(list);

    let expanded = quote! {
        &[#(#ints),*]
    };

    expanded
}

fn int_once(list: &MetaList) -> proc_macro2::TokenStream {
    let ints = get_int_literals(list);

    if ints.len() != 1 {
        panic!("Expected a single {integer}");
    }

    let int = &ints[0];

    let expanded = quote! {
        #int
    };

    expanded
}

fn get_int_literals(list: &MetaList) -> Vec<syn::LitInt> {
    let mut ints: Vec<syn::LitInt> = Vec::new();
    let mut iter = list.nested.iter();

    while let Some(NestedMeta::Literal(Lit::Int(int))) = iter.next() {
        ints.push(int.clone());
    }

    ints
}

fn get_func_ident(list: &MetaList) -> Ident {
    if let NestedMeta::Meta(Meta::Word(ident)) = list.nested.iter().next().unwrap() {
        ident.clone()
    } else {
        panic!("Invalid attribute func name")
    }
}

// fn int_range(list: &MetaList) -> Ident {

// }

#[proc_macro]
pub fn generate_structnom(input: TokenStream) -> TokenStream {
    let endian = parse_macro_input!(input as Endian);

    let byte_impl = gen_byte_impl(endian);
    let hword_impl = gen_hword_impl(endian);
    let word_impl = gen_word_impl(endian);
    let long_impl = gen_long_impl(endian);
    let float_impl = gen_float_impl(endian);
    let vec_impl = gen_vec_impl(endian);
    let option_impl = gen_option_impl();

    let expanded = quote! {
        pub trait StructNom {
            fn nom(input: &[u8]) -> nom::IResult<&[u8], Self> where Self: Sized;
        }

        #byte_impl
        #hword_impl
        #word_impl
        #long_impl
        #float_impl
        #vec_impl
        #option_impl
    };

    // log::debug!("StructNom Derivation {}", expanded);

    TokenStream::from(expanded)
}

fn get_path(attr: &Attribute) -> syn::Path {
    if !attr.path.is_ident("parser") {
        panic!("Expectred \"parser\"");
    }

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
