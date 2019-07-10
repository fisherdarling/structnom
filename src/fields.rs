use syn::{Fields, FieldsNamed, FieldsUnnamed, Ident};

use quote::quote;

use crate::attrs;

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
            // Fields::Unnamed(unnamed) => self.gen_unnamed_parser(unnamed),
            // Fields::Unit => self.gen_unit_parser(),
            _ => panic!(),
        };

        field_parser
    }

    fn gen_named_parser(&self, fields: &FieldsNamed) -> proc_macro2::TokenStream {
        let mut idents = Vec::new();
        let mut parsers = Vec::new();

        for field in fields.named.iter() {
            let config: attrs::Field = attrs::Field::from_field(field);

            

        }

        let name = self.gen_name();

        quote! {
            // do_parse! (
            //     #(#parsers)*
            //     (#name { #(#idents),* })
            // )
        }
    }

    // fn gen_unnamed_parser(&self, fields: &FieldsUnnamed) -> proc_macro2::TokenStream {
    //     let mut idents = Vec::new();
    //     let mut parsers = Vec::new();

    //     for (i, field) in fields.unnamed.iter().enumerate() {
    //         let snom_args: Vec<_> = field
    //             .attrs
    //             .iter()
    //             .map(SnomArg::parse)
    //             .map(|v| v.unwrap())
    //             .flatten()
    //             .collect();

    //         let _ = snom_args.iter().find_map(SnomArg::match_arg);
    //         let value_arg = snom_args.iter().find_map(SnomArg::value_arg);
    //         let effect_args: Vec<_> = snom_args.iter().filter_map(SnomArg::effect_arg).collect();

    //         // println!("*** EffectArgs Outer {:?}", effect_args);

    //         let field_name = format!("f_{}", i);
    //         let field_ident = Ident::new(&field_name, field.ident.span());
    //         let ty = &field.ty;

    //         let parser =
    //             FieldsGen::gen_field_parser(&field_ident, ty, &value_arg, effect_args.as_slice());

    //         idents.push(field_ident);
    //         parsers.push(parser);
    //     }

    //     let name = self.gen_name();

    //     quote! {
    //         do_parse! (
    //             #(#parsers)*
    //             (#name ( #(#idents),* ))
    //         )
    //     }
    // }

    // pub fn gen_field_parser(
    //     ident: &Ident,
    //     ty: &syn::Type,
    //     value_arg: &Option<&ValueArg>,
    //     effect_args: &[&EffectArg],
    // ) -> proc_macro2::TokenStream {
    //     let field_span = ident.span();

    //     match value_arg {
    //         Some(ValueArg::Parser { value, .. }) => {
    //             quote_spanned!(field_span=> #ident: call!(#value) >>)
    //         }
    //         Some(ValueArg::Skip { .. }) => {
    //             quote_spanned!(field_span=> #ident: value!(Default::default()) >>)
    //         }
    //         Some(ValueArg::Bits { count, .. }) => {
    //                 quote_spanned!(field_span=> #ident: bits!(take_bits!(#ty, #count)) >>)
    //         }
    //         Some(ValueArg::TagBits { count, pattern, .. }) => {
    //             quote_spanned!(field_span=> #ident: bits!(tag_bits!(#ty, #count, #pattern)) >>)
    //         }
    //         Some(p) => {
    //             let error = format!("Unimplemented field parser: {:?}", p);
    //             quote_spanned!(field_span=> #ident: value!(compile_error!(#error)) >>)
    //         }
    //         None => {
    //             // println!("Effect Args: {:?}", effect_args);

    //             quote_spanned! {field_span=>
    //                 #(#effect_args >>)*
    //                 #ident: call!(<#ty>::nom) >>
    //             }
    //         }
    //     }
    // }

    // fn gen_unit_parser(&self) -> proc_macro2::TokenStream {
    //     let args: Vec<_> = self.fields.iter().map(|f| f.attrs.as_slice()).flatten().map(SnomArg::parse).flatten().flatten().collect();
    //     let value_arg = args.iter().find_map(SnomArg::value_arg);

    //     // println!("UnitField ValueArgs: {:?}", value_arg);

    //     match value_arg {
    //         Some(ValueArg::Parser { value, ..}) => {
    //             quote! { call!(#value) }
    //         },
    //         Some(_) => quote! { compile_error!("Unimplemented field value argument") },
    //         None => {
    //             let name = self.gen_name();
    //             quote! { value!(#name) }
    //         }
    //     }
    // }

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
