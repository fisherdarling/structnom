// use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
// use syn::parse::{Lookahead1, Parse, ParseBuffer, ParseStream, Peek};
// use syn::{parenthesized, punctuated::Punctuated, Attribute, LitInt, LitStr, Meta, Token, token::Token};

use syn::{
    parse2, parse_macro_input, spanned::Spanned, AttrStyle, Attribute, Data, DataEnum, DataStruct,
    DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed, Generics, Ident, Lit, LitInt, Meta,
    MetaList, MetaNameValue, NestedMeta, Result as SynResult, Variant,
};

use crate::attr::*;

// let expanded = gen_struct_impl(name, &input.attrs, generics, data);

#[derive(Debug, Clone)]
pub enum EnumState {
    Start(LitInt),
    Middle(LitInt),
    End(LitInt),
    None,
}

pub struct EnumGen {
    name: Ident,
    args: Vec<SnomArg>,
    generics: Generics,
    data: DataEnum,
    state: EnumState,
}

impl EnumGen {
    pub fn new(
        name: Ident,
        enum_attrs: Vec<Attribute>,
        generics: Generics,
        data: DataEnum,
    ) -> EnumGen {
        let args: Vec<_> = enum_attrs
            .iter()
            .map(SnomArg::parse)
            .filter_map(Result::ok)
            .flatten()
            .collect();

        println!("Enum Args: {:?}", args);

        EnumGen {
            name,
            args,
            generics,
            data,
            state: EnumState::None,
        }
    }

    pub fn gen_impl(&mut self) -> proc_macro2::TokenStream {
        let mut parsers: Vec<_> = Vec::new();

        for variant in self.data.variants.clone() {
            let parser = self.gen_variant_parser(variant);

            if !parser.is_empty() {
                parsers.push(parser);
            }
        }

        let name = &self.name;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
        let switch_func = self.get_switch_func();

        let span = name.span();
        let expanded = quote_spanned! {span=>
            impl #impl_generics crate::StructNom for #name #ty_generics #where_clause {
                fn nom(input: &[u8]) -> nom::IResult<&[u8], Self> {
                    do_parse!(input,
                    val: switch!(call!(#switch_func),
                            #(#parsers)|*
                        ) >>
                        (val))
                }
            }
        };

        expanded
    }

    fn get_switch_func(&self) -> proc_macro2::TokenStream {
        let possible: Vec<&ValueArg> = self.args.iter().filter_map(SnomArg::value_arg).collect();

        println!("Possible Attrs {:?}", possible);

        if let Some(ValueArg::Switch { value, .. }) = possible.get(0) {
            quote! { #value }
        } else {
            let span = self.name.span();
            quote_spanned!(span=> compile_error!("Enums must have a defined switch parser."))
        }
    }

    pub fn gen_variant_parser(&mut self, variant: Variant) -> proc_macro2::TokenStream {
        let variant_name = &variant.ident;

        let snom_args: Vec<_> = variant
            .attrs
            .iter()
            .map(SnomArg::parse)
            .flatten()
            .flatten()
            .collect();

        let mut match_arg = snom_args.iter().find_map(SnomArg::match_arg);
        let mut value_arg = snom_args.iter().find_map(SnomArg::value_arg);
        let mut post_parsers: Vec<_> = snom_args.iter().filter_map(SnomArg::effect_arg).collect();

        assert!(post_parsers.is_empty());

        //         Parser {
        //     parser_token: kw::parser,
        //     eq_token: Token![=],
        //     value: syn::Path,
        // },
        // Skip {
        //     skip_token: kw::skip,
        // },
        // Iter {
        //     iter_token: kw::iter,
        // },
        let match_arm = self.handle_match_arm(match_arg, variant.span());
        let variant_span = variant.span();

        match value_arg {
            Some(ValueArg::Parser { value, .. }) => {
                return quote_spanned!(variant_span=> #match_arm => call!(#value))
            }
            Some(ValueArg::Skip { .. }) => {
                println!("Empty: {:#?}", quote_spanned!(variant_span=> ));

                return quote_spanned!(variant_span=> )
            }
            Some(_) => {
                return quote_spanned!(variant_span=> compile_error!("Unimplemented variant attribute."))
            }
            None => {}
        }

        let field_gen = FieldsGen::new(&self.name, Some(&variant.ident), &variant.fields);
        let field_parser = field_gen.gen_parser();

        let expanded = quote_spanned! {variant_span=>
            #match_arm => #field_parser
        };

        expanded
    }

    fn handle_range(&mut self, range: &RangeArg) -> proc_macro2::TokenStream {
        println!("Range State: {:?}, Range Arg: {:?}", self.state, range);

        match range {
            RangeArg::Start { value, .. } => {
                if let EnumState::None = self.state.clone() {
                    self.state = EnumState::Start(value.clone());

                    quote! { #value }
                } else {
                    quote! { compile_error!("Invalid range(start ...), another range is already in progress.")}
                }
            }
            RangeArg::Skip { value, .. } => {
                if let EnumState::Start(prev) | EnumState::Middle(prev) = self.state.clone() {
                    let value = value
                        .clone()
                        .unwrap_or(LitInt::new(1, prev.suffix(), prev.span()));

                    let new_lit = LitInt::new(
                        prev.value() + value.value() + 1,
                        value.suffix(),
                        value.span(),
                    );
                    self.state = EnumState::Middle(new_lit.clone());

                    quote! { #new_lit }
                } else {
                    quote! { compile_error!("Invalid range(skip ...), a range must be started before one can be skipped.") }
                }
            }
            RangeArg::End { value, .. } => {
                if let EnumState::Middle(prev) | EnumState::Start(prev) = self.state.clone() {
                    if value.value() == prev.value() + 1 {
                        self.state = EnumState::None;

                        quote! { #value }
                    } else {
                        quote! { compile_error!("Invalid range(end ...), the ending value must be equal to the previous value + 1.")}
                    }
                } else {
                    quote! { compile_error!("Invalid range(end ...), a range must only end after one has been started.")}
                }
            }
        }
    }

    fn handle_match_arm(&mut self, match_arm: Option<&MatchArg>, span: proc_macro2::Span) -> proc_macro2::TokenStream {
        match match_arm {
            Some(MatchArg::Range(range)) => self.handle_range(range),
            Some(MatchArg::Val { value, .. }) => quote! { #value },
            Some(MatchArg::Values { values, .. }) => quote! { &[#(#values),*] },
            None => {
                if let EnumState::Start(ref mut lit) | EnumState::Middle(ref mut lit) = self.state {
                    let new_lit = LitInt::new(lit.value() + 1, lit.suffix(), lit.span());
                    *lit = new_lit.clone();

                    quote_spanned! (span=> #new_lit )
                } else {
                    quote_spanned! (span=> compile_error!("StructNom requires all enum variants to have something to match on.") )
                }
            }
        }
    }

    // fn gen_parser(
    //     &mut self,
    //     value_arg: Option<&ValueArg>,
    //     match_arm: Option<&MatchArg>,
    //     post_parsers: Vec<&EffectArg>,
    // ) {
    //     match value_arg {
    //         Some(ValueArg::Parser { value, .. }) => {

    //         }
    //         _ => {}
    //     }
    // }
}

#[derive(Debug, Clone)]
pub struct StructGen {
    name: Ident,
    args: Vec<SnomArg>,
    generics: Generics,
    data: DataStruct,
    // state: EnumState,
}

impl StructGen {
    pub fn new(
        name: Ident,
        enum_attrs: Vec<Attribute>,
        generics: Generics,
        data: DataStruct,
    ) -> StructGen {
        let args: Vec<_> = enum_attrs
            .iter()
            .map(SnomArg::parse)
            .filter_map(Result::ok)
            .flatten()
            .collect();

        println!("Struct Args: {:?}", args);

        StructGen {
            name,
            args,
            generics,
            data,
        }
    }

    pub fn gen_impl(&mut self) -> proc_macro2::TokenStream {
        let value_arg = self.args.iter().find_map(SnomArg::value_arg);
        let field_parser = match value_arg {
            Some(ValueArg::Parser { value, .. }) => quote! { call!(#value) },
            Some(_) => quote! { compile_error!("Unimplemented StructNom ValueArg for Struct, {:?}", self.name) },
            None => {
                let field_gen = FieldsGen::new(&self.name, None, &self.data.fields);
                field_gen.gen_parser()        
            }
        };

        let name = &self.name;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let span = self.name.span();
        let expanded = quote_spanned! {span=>
            impl #impl_generics crate::StructNom for #name #ty_generics #where_clause {
                fn nom(input: &[u8]) -> nom::IResult<&[u8], Self> {
                    do_parse!(input,
                        val: #field_parser >>
                        (val))
                }
            }
        };

        expanded
    }


}

#[derive(Debug, Clone)]
pub struct FieldsGen<'a> {
    name: &'a Ident,
    variant: Option<&'a Ident>,
    fields: &'a Fields,
}

impl<'a> FieldsGen<'a> {
    pub fn new(name: &'a Ident, variant: Option<&'a Ident>, fields: &'a Fields) -> FieldsGen<'a> {
        FieldsGen {
            name,
            variant,
            fields,
        }
    }

    pub fn gen_parser(&self) -> proc_macro2::TokenStream {
        let field_parser = match &self.fields {
            Fields::Named(named) => self.gen_named_parser(named),
            Fields::Unnamed(unnamed) => self.gen_unnamed_parser(unnamed),
            Fields::Unit => self.gen_unit_parser(),
        };

        field_parser
    }

    fn gen_named_parser(&self, fields: &FieldsNamed) -> proc_macro2::TokenStream {
        let mut idents = Vec::new();
        let mut parsers = Vec::new();

        for field in fields.named.iter() {
            let snom_args: Vec<_> = field
                .attrs
                .iter()
                .map(SnomArg::parse)
                .flatten()
                .flatten()
                .collect();

            let _ = snom_args.iter().find_map(SnomArg::match_arg);
            let value_arg = snom_args.iter().find_map(SnomArg::value_arg);
            let effect_args: Vec<_> = snom_args.iter().filter_map(SnomArg::effect_arg).collect();

            // println!("*** EffectArgs Outer {:?}", effect_args);


            let field_ident = field.ident.clone().expect("Named Fields must be named");
            let ty = &field.ty;

            // println!("ValueArg: {:?}", value_arg);

            let parser =
                FieldsGen::gen_field_parser(&field_ident, ty, &value_arg, effect_args.as_slice());

            idents.push(field_ident);
            parsers.push(parser);
        }

        let name = self.gen_name();

        quote! {
            do_parse! (
                #(#parsers)*
                (#name { #(#idents),* })
            )
        }
    }

    fn gen_unnamed_parser(&self, fields: &FieldsUnnamed) -> proc_macro2::TokenStream {
        let mut idents = Vec::new();
        let mut parsers = Vec::new();

        for (i, field) in fields.unnamed.iter().enumerate() {
            let snom_args: Vec<_> = field
                .attrs
                .iter()
                .map(SnomArg::parse)
                .map(|v| v.unwrap())
                .flatten()
                .collect();

            let _ = snom_args.iter().find_map(SnomArg::match_arg);
            let value_arg = snom_args.iter().find_map(SnomArg::value_arg);
            let effect_args: Vec<_> = snom_args.iter().filter_map(SnomArg::effect_arg).collect();

            // println!("*** EffectArgs Outer {:?}", effect_args);

            let field_name = format!("f_{}", i);
            let field_ident = Ident::new(&field_name, field.ident.span());
            let ty = &field.ty;

            let parser =
                FieldsGen::gen_field_parser(&field_ident, ty, &value_arg, effect_args.as_slice());

            idents.push(field_ident);
            parsers.push(parser);
        }

        let name = self.gen_name();

        quote! {
            do_parse! (
                #(#parsers)*
                (#name ( #(#idents),* ))
            )
        }
    }

    pub fn gen_field_parser(
        ident: &Ident,
        ty: &syn::Type,
        value_arg: &Option<&ValueArg>,
        effect_args: &[&EffectArg],
    ) -> proc_macro2::TokenStream {
        let field_span = ident.span();

        match value_arg {
            Some(ValueArg::Parser { value, .. }) => {
                quote_spanned!(field_span=> #ident: call!(#value) >>)
            }
            Some(ValueArg::Skip { .. }) => {
                quote_spanned!(field_span=> #ident: value!(Default::default()) >>)
            }
            Some(ValueArg::Bits { count, .. }) => {
                    quote_spanned!(field_span=> #ident: bits!(take_bits!(#ty, #count)) >>)
            }
            Some(ValueArg::TagBits { count, pattern, .. }) => {
                quote_spanned!(field_span=> #ident: bits!(tag_bits!(#ty, #count, #pattern)) >>)
                
            }
            Some(p) => {
                let error = format!("Unimplemented field parser: {:?}", p);
                quote_spanned!(field_span=> #ident: value!(compile_error!(#error)) >>)
            }
            None => {
                // println!("Effect Args: {:?}", effect_args);

                quote_spanned! {field_span=>
                    #(#effect_args >>)*
                    #ident: call!(<#ty>::nom) >>
                }
            }
        }
    }

    fn gen_unit_parser(&self) -> proc_macro2::TokenStream {
        let args: Vec<_> = self.fields.iter().map(|f| f.attrs.as_slice()).flatten().map(SnomArg::parse).flatten().flatten().collect();
        let value_arg = args.iter().find_map(SnomArg::value_arg);

        // println!("UnitField ValueArgs: {:?}", value_arg);

        match value_arg {
            Some(ValueArg::Parser { value, ..}) => {
                quote! { call!(#value) }
            },
            Some(_) => quote! { compile_error!("Unimplemented field value argument") },
            None => {
                let name = self.gen_name();
                quote! { value!(#name) }
            }
        }
    }

    fn gen_name(&self) -> proc_macro2::TokenStream {
        if let Some(ref ident) = self.variant {
            let name = self.name;
            quote! { #name::#ident }
        } else {
            let name = self.name;
            quote! { #name }
        }
    }
}

// impl crate::StructNom for Instr {
//     fn nom(input: &[u8]) -> nom::IResult<&[u8], Self> {
//         do_parse!(
//             input,
//             val:
//                 switch!(call!(le_u8),
//                     1 => value!(Instr::Nop) |
//                     compile_error!("StructNom requires all enum variants to have something to match on.") => value ! ( Instr :: If ) | compile_error ! ( "StructNom requires all enum variants to have something to match on." ) => value ! ( Instr :: Add ) | compile_error ! ( "Invalid range(skip ...), a range must be started before one can be skipped." ) => value ! ( Instr :: Sub ) | compile_error ! ( "Invalid range(skip ...), a range must be started before one can be skipped." ) => value ! ( Instr :: Skip3 ) | compile_error ! ( "Invalid range(end ...), the ending value must be equal to the previous value + 1." ) => value ! ( Instr :: Equal ) | compile_error ! ( "StructNom requires all enum variants to have something to match on." ) => value ! ( Instr :: Another ) )
//                 >> (val)
//         )
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StructNom;
    use syn::parse::Parse;

    fn get_enum_data(
        code: proc_macro2::TokenStream,
    ) -> (Ident, Vec<Attribute>, Generics, DataEnum) {
        let input = parse2::<syn::DeriveInput>(code).unwrap();

        let name = &input.ident;
        let attrs = input.attrs;
        let generics = input.generics;

        if let Data::Enum(data) = input.data {
            return (name.clone(), attrs.clone(), generics, data);
        } else {
            panic!("Not an enum")
        }
    }

    fn get_struct_data(
        code: proc_macro2::TokenStream,
    ) -> (Ident, Vec<Attribute>, Generics, DataStruct) {
        let input = parse2::<syn::DeriveInput>(code).unwrap();

        let name = &input.ident;
        let attrs = input.attrs;
        let generics = input.generics;

        if let Data::Struct(data) = input.data {
            return (name.clone(), attrs.clone(), generics, data);
        } else {
            panic!("Not an enum")
        }
    }

    #[test]
    fn enum_gen() {
        // let code: proc_macro2::TokenStream = syn::parse_quote! {
        //     #[derive(StructNom)]
        //     #[snom(switch = le_u8)]
        //     pub enum MyEnum {
        //         #[snom(values(0x04, 0x03, 5, 10))]
        //         First { thing_one: u32, memes: Vec<u8> },
        //         #[snom(val = 0x04)]
        //         Unnamed(u64, Expr),
        //     }
        // };

        let code: proc_macro2::TokenStream = syn::parse_quote! {
            #[derive(StructNom)]
            #[snom(switch = le_u8)]
            pub enum Instr {
                #[snom(range(start = 1))]
                Nop, // 1
                If,  // 2
                #[snom(parser = crate::my_parser)]
                Add, // 3
                #[snom(range(skip))]
                Sub, // 5
                #[snom(val = 0x04)]
                Mul, // 
                #[snom(range(skip = 3))]
                Skip3, // 9
                #[snom(range(end = 10))]
                Equal, // 10
                #[snom(val = 0x0F)]
                Another, // 15
            }
        };

        let (name, attrs, generics, data) = get_enum_data(code);

        let mut gen = EnumGen::new(name, attrs, generics, data);

        let enum_impl = gen.gen_impl();

        println!("{}", enum_impl);

        panic!()
    }

    #[test]
    fn struct_gen() {
        let code: proc_macro2::TokenStream = syn::parse_quote! {
            #[derive(StructNom)]
            pub struct MyStruct(#[snom(tag(SOME_SLICE))] u32, Expr);
        };

        let (name, attrs, generics, data) = get_struct_data(code);

        let mut gen = StructGen::new(name, attrs, generics, data);
        let struct_impl = gen.gen_impl();

        println!("{}", struct_impl);

        panic!();
    }
}

// pub fn gen_enum_field(field: &Field, variant: &Variant, state: &mut EnumState) -> proc_macro2::TokenStream {
//     unimplemented!()
// }
