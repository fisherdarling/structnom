#![recursion_limit="128"]

extern crate proc_macro;

use crate::proc_macro::TokenStream;

use std::convert::TryInto;

use quote::{quote, ToTokens};
use syn::{
    parse2, parse_macro_input, AttrStyle, Attribute, Data, DataEnum, DataStruct, DeriveInput,
    Field, Fields, FieldsNamed, FieldsUnnamed, Generics, Ident, Lit, LitInt, Meta, MetaList,
    NestedMeta, Variant,
};

use proc_macro2::TokenTree;

#[proc_macro_derive(Nom)]
pub fn nom_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    // let attrs = &input.attrs;
    let generics = input.generics;

    match input.data {
        Data::Struct(data) => {
            let expanded = gen_struct_impl(data, generics, name);

            TokenStream::from(expanded)
        }
        Data::Enum(data) => {
            let expanded = gen_enum_impl(name, &input.attrs, generics, data);

            TokenStream::from(expanded)
        }
        Data::Union(_) => panic!("Union types are not supported yet."),
    }
}

fn gen_struct_impl(data: DataStruct, generics: Generics, name: &Ident) -> proc_macro2::TokenStream {
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();;

    let fields = data.fields;

    let fields_do_parse = match fields {
        Fields::Named(ref fields) => gen_named_fields(name, fields),
        Fields::Unnamed(ref fields) => gen_unnamed_fields(name, fields),
        Fields::Unit => quote! { Ok((input, Self)) },
        // _ => panic!("Fields Unnamed"),
    };

    let expanded = quote! {
        impl #impl_generics structnom::Nom for #name #ty_generics #where_clause {
            fn nom(input: &[u8]) -> nom::IResult<&[u8], Self> {
                let res = #fields_do_parse;

                res
            }
        }
    };

    println!("Gen Struct Impl {}", expanded);

    expanded
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

    println!("Enum Attr Kind: {}", kind);

    let switch_parser = get_func_ident(&list);

    println!("Switch Parser: {}", switch_parser.to_string());

    let match_arms: Vec<_> = data.variants.iter().map(get_match_arm).collect();

    let variant_parsers: Vec<_> = data
        .variants
        .iter()
        .map(|variant| match variant.fields {
            Fields::Named(ref named) => gen_enum_named_fields(name, &variant, named),
            Fields::Unnamed(ref unnamed) => gen_enum_unnamed_fields(name, &variant, unnamed),
            Fields::Unit => {
                println!("Unit Variant Field");
                let var_ident = &variant.ident;

                quote! {
                    value!(#name::#var_ident)
                }
            }
        })
        .collect();

    let expanded = quote! {
        impl #impl_generics structnom::Nom for #name #ty_generics #where_clause {
            fn nom(input: &[u8]) -> nom::IResult<&[u8], Self> {
                do_parse!(input,
                val: switch!(#switch_parser,
                        #(#match_arms => #variant_parsers)|*
                    ) >>
                    (val)
                )
            }
        }
    };

    println!("{}", expanded);

    expanded
}

fn get_match_arm(variant: &Variant) -> proc_macro2::TokenStream {
    let attr = &variant
        .attrs
        .get(0)
        .expect("Expected attribute for match arm.")
        .clone();

    let meta = attr
        .parse_meta()
        .expect("Unable to parse attribute metadata in match arm.");

    let list = match meta {
        Meta::List(list) => list,
        _ => unimplemented!("Get match arm => unknown metadata."),
    };

    let kind = list.ident.to_string();

    let expanded = match kind.as_ref() {
        "byte" => int_once(&list),
        "bytes" => int_slice(&list),
        _ => panic!("Match arms must be in the form of #[byte(LitInt)] to match a single byte, or #[bytes(LitInt, LitInt, ...)] to match a slice of integers."),
    };

    expanded
}

fn gen_named_fields(name: &Ident, fields: &FieldsNamed) -> proc_macro2::TokenStream {
    let idents: Vec<_> = fields
        .named
        .iter()
        .map(|f| f.ident.clone().unwrap())
        .collect();
    let idents2 = idents.clone();

    // let types: Vec<_> = fields.named.iter().map(|f| f.ty.clone()).collect();
    let parsers: Vec<_> = fields.named.iter().map(gen_field_parser).collect();

    let expanded = quote! {
        nom::do_parse!(input,
            #(
                #idents: #parsers >>
            )*
            (#name { #(#idents2),* } )
        )
    };

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

    // println!("Parsers: {:#?}", parsers);

    let expanded = quote! {
        do_parse!(
            #(
                #idents: #parsers >>
            )*
            (#name::#variant_ident ( #(#idents2),* ) )
        )
    };

    // println!("Expanded: {}", expanded);

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

    // println!("Parsers: {:#?}", parsers);

    let expanded = quote! {
        nom::do_parse!(input,
            #(
                #idents: #parsers >>
            )*
            (#name ( #(#idents2),* ) )
        )
    };

    // println!("Expanded: {}", expanded);

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
        let meta = &field.attrs[0].parse_meta().unwrap();

        let parser = match meta {
            Meta::List(list) => gen_parser_meta_list(list, field),
            m => unimplemented!("Meta"),
        };

        parser
    }
}

fn gen_parser_meta_list(list: &MetaList, field: &Field) -> proc_macro2::TokenStream {
    let kind: String = list.ident.to_string();

    match kind.as_str() {
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
        "parser" => {
            let func = get_func_ident(&list);

            let expanded = quote! {
                call!(#func)
            };

            expanded
        }
        _ => unimplemented!("Unimplemented field attribute"),
    }
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

use syn::parse::{Parse, ParseStream};


enum Endian {
    Little,
    Big,
}

impl Parse for Endian {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(Ident) {
            let mut kind = input.parse::<Ident>()?.to_string();
            kind.make_ascii_lowercase();
         
            match kind.as_ref() {
                "big" => Ok(Endian::Big),
                "little" => Ok(Endian::Little),
                _ => panic!("Unsupported Argument"),
            }
        } else {
            Err(lookahead.error())
        }
    }
}

#[proc_macro]
pub fn generate_nom_trait(input: TokenStream) -> TokenStream {
    let endian = parse_macro_input!(input as Endian);

    let func_prefix = match endian {
        Endian::Little => quote! {
            le
        },
        Endian::Big => quote! {
            be
        }
    };

    let expanded = quote! {
        pub trait Nom {
            fn nom(input: &[u8]) -> IResult<&[u8], Self>
                where Self: Sized;
        }

        

        impl Nom for u32 {
            fn nom(input: &[u8]) -> IResult<&[u8], Self> {
                let (input, res) = #func_prefix_u32(input)?;

                Ok((input, res))
            }
        }

        impl<T: Nom> Nom for Vec<T> {
            fn nom(input: &[u8]) -> IResult<&[u8], Self> {
                parse_vec(input)
            }
        }

        impl<T: Nom> Nom for Option<T> {
            fn nom(input: &[u8]) -> IResult<&[u8], Self> {
                opt!(input, complete!(T::nom))
            }
        }

        impl Nom for String {
            fn nom(input: &[u8]) -> IResult<&[u8], Self> {
                let (input, bytes) = <Vec<u8>>::nom(input)?;

                Ok((input, String::from_utf8(bytes).unwrap()))
            }
        }

        pub fn parse_vec<T: Nom>(data: &[u8]) -> IResult<&[u8], Vec<T>> {
            let (input, length) = #func_prefix_u8(data)?;

            // println!("Parsing vec of length {}", length);

            count!(input, Nom::nom, length as usize)
        }
    };


    TokenStream::from(expanded)
}

fn gen_8_impl(endian: Endian) -> proc_macro2::TokenStream {
    quote!{
        impl Nom for u8 {
            fn nom(input: &[u8]) -> IResult<&[u8], Self> {
                let (input, res) = #func_prefix_u8(input)?;

                Ok((input, res))
            }
        }
    }
}

fn gen_32_impl(endian: Endian) -> proc_macro2::TokenStream {
    quote!{
        
    }
}
fn gen_vec_impl(endian: Endian) -> proc_macro2::TokenStream {
    quote!{
        
    }
}
fn gen_opt_impl(endian: Endian) -> proc_macro2::TokenStream {
    quote!{
        
    }
}
fn gen_string_impl(endian: Endian) -> proc_macro2::TokenStream {
    quote!{
        
    }
}