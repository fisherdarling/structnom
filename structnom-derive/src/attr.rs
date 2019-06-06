//! Parsing logic for StructNom attributes.

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Lookahead1, Parse, ParseBuffer, ParseStream, Peek};
use syn::{
    parenthesized, punctuated::Punctuated, token::Token, Attribute, LitInt, LitStr, Meta, Token,
};
// use syn::lookahead::TokenMarker;

use syn::Result as SynResult;

#[derive(Debug, Clone)]
pub enum SnomArg {
    Match(MatchArg),
    Parser(ParserArg),
    Effect(EffectArg),
    None,
}

impl SnomArg {
    pub fn parse(attr: Attribute) -> SynResult<Option<Self>> {
        if is_structnom_attr(&attr) {
            Ok(Some(syn::parse2::<SnomArg>(attr.tts)?))
        } else {
            Ok(None)
        }
    }
}

impl Parse for SnomArg {
    fn parse(input: ParseStream) -> SynResult<Self> {
        // println!("SnomArg Input: {:#?}", input);
        // We're looking at the tts of a snom attribute:
        //
        // #[snom(some_tts)]
        //       ^^^^^^^^^^
        //
        // This inclues the surrounding parentheses.
        // Therefore we first remove them using the
        // parenthesized! macro.
        // let content;
        // parenthesized!(content in input);
        let input = pop_parens(input)?;

        let lookahead = input.lookahead1();

        // println!("SnomArg: input {:#?}", input);

        if looking_at_match(&lookahead) {
            Ok(SnomArg::Match(input.parse()?))
        } else if looking_at_parser(&lookahead) {
            Ok(SnomArg::Parser(input.parse()?))
        } else if looking_at_effect(&lookahead) {
            Ok(SnomArg::Effect(input.parse()?))
        } else {
            Ok(SnomArg::None)
        }
    }
}

pub fn looking_at_match(lookahead: &Lookahead1) -> bool {
    lookahead.peek(kw::range) || lookahead.peek(kw::val) || lookahead.peek(kw::values)
}

pub fn looking_at_parser(lookahead: &Lookahead1) -> bool {
    lookahead.peek(kw::parser) || lookahead.peek(kw::skip) || lookahead.peek(kw::iter)
}

pub fn looking_at_effect(lookahead: &Lookahead1) -> bool {
    lookahead.peek(kw::debug)
}

#[derive(Debug, Clone)]
pub enum MatchArg {
    Range(RangeArg),
    Val {
        val_token: kw::val,
        eq_token: Token![=],
        value: LitInt,
    },
    Values {
        values_token: kw::values,
        paren_token: syn::token::Paren,
        values: Punctuated<LitInt, Token![,]>,
    },
}

impl Parse for MatchArg {
    fn parse(input: ParseStream) -> SynResult<Self> {
        // println!("MatchArg Input: {:#?}", input);

        let lookahead = input.lookahead1();

        if lookahead.peek(kw::range) {
            let _range_token = input.parse::<kw::range>()?;
            let input = pop_parens(input)?;
            Ok(MatchArg::Range(input.parse()?))
        } else if lookahead.peek(kw::val) {
            Ok(MatchArg::Val {
                val_token: input.parse()?,
                eq_token: input.parse()?,
                value: input.parse()?,
            })
        } else if lookahead.peek(kw::values) {
            let content;
            Ok(MatchArg::Values {
                values_token: input.parse()?,
                paren_token: parenthesized!(content in input),
                values: content.parse_terminated(LitInt::parse)?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug, Clone)]
pub enum RangeArg {
    Start {
        start_token: kw::start,
        eq_token: Token![=],
        value: LitInt,
    },
    Skip {
        skip_token: kw::skip,
        eq_token: Option<Token![=]>,
        value: Option<LitInt>,
    },
    End {
        end_token: kw::end,
        eq_token: Token![=],
        value: LitInt,
    },
}

impl Parse for RangeArg {
    fn parse(input: ParseStream) -> SynResult<Self> {
        // println!("RangeArg Input: {:#?}", input);

        let lookahead = input.lookahead1();

        if lookahead.peek(kw::start) {
            Ok(RangeArg::Start {
                start_token: input.parse()?,
                eq_token: input.parse()?,
                value: input.parse()?,
            })
        } else if lookahead.peek(kw::skip) {
            Ok(RangeArg::Skip {
                skip_token: input.parse()?,
                eq_token: input.parse()?,
                value: input.parse()?,
            })
        } else if lookahead.peek(kw::end) {
            Ok(RangeArg::End {
                end_token: input.parse()?,
                eq_token: input.parse()?,
                value: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug, Clone)]
pub enum ParserArg {
    Parser {
        parser_token: kw::parser,
        eq_token: Token![=],
        value: LitStr,
    },
    Skip {
        skip_token: kw::skip,
    },
    Iter {
        iter_token: kw::iter,
    },
}

impl Parse for ParserArg {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(kw::parser) {
            Ok(ParserArg::Parser {
                parser_token: input.parse()?,
                eq_token: input.parse()?,
                value: input.parse()?,
            })
        } else if lookahead.peek(kw::skip) {
            Ok(ParserArg::Skip {
                skip_token: input.parse()?,
            })
        } else if lookahead.peek(kw::iter) {
            Ok(ParserArg::Iter {
                iter_token: input.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug, Clone)]
pub enum EffectArg {
    Debug {
        debug_token: kw::debug,
        eq_token: Option<Token![=]>,
        value: Option<LitStr>,
    },
}

impl Parse for EffectArg {
    fn parse(input: ParseStream) -> SynResult<Self> {
        Ok(EffectArg::Debug {
            debug_token: input.parse()?,
            eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

pub fn is_structnom_attr(attr: &syn::Attribute) -> bool {
    attr.path.segments.len() == 1 && attr.path.segments[0].ident == "snom"
}

pub fn pop_parens(input: ParseStream) -> SynResult<ParseBuffer> {
    let content;
    parenthesized!(content in input);
    Ok(content)
}

mod kw {
    use syn::custom_keyword;

    custom_keyword!(snom);

    custom_keyword!(range);
    custom_keyword!(start);
    custom_keyword!(skip);
    custom_keyword!(end);

    custom_keyword!(val);
    custom_keyword!(values);
    custom_keyword!(slice);

    custom_keyword!(parser);
    custom_keyword!(iter);
    // custom_keyword!(complete);

    custom_keyword!(debug);
}

// pub enum Either<A: Parse + Token + Peek, B: Parse + Token + Peek> {
//     Left(A),
//     Right(B),
// }

// impl<A, B> Parse for Either<A, B>
// where
//     A: Token + Parse,
//     B: Token + Parse
//   {
//     fn parse(input: ParseStream) -> SynResult<Self> {
//         let lookahead = input.lookahead1();

//         if lookahead.peek(A::Token) {
//             Ok(Either::Left(input.parse::<A>()?))
//         } else if lookahead.peek(B::Token) {
//             Ok(Either::Right(input.parse::<B>()?))
//         } else {
//             Err(lookahead.error())
//         }
//     }
// }

/// Attribute usage:
///
/// #[derive(StructNom)]
/// pub enum Instr {
///     #[snom(range(start = 1))]
///     Nop, // 1
///     If,  // 2
///     #[snom(parser = "crate::my_parser")]
///     Add, // 3
///     #[snom(range(skip))]
///     Sub, // 5
///     #[snom(range(skip = 3))]
///     Skip3, // 9
///     #[snom(range(end = 10))]
///     Equal, // 10
///     #[snom(val = "0x0F")]
///     Another, // 15
/// }
///
/// #[derive(StructNom)]
/// pub struct Example<T> {
///     #[snom(debug = "0x{:x?}")]
///     #[snom(parser = "crate::leb_u32")]
///     foo: u32,
///     #[snom(skip)]
///     bar: Vec<T>,
///     baz: String,
///     #[snom(debug)]
///     #[snom(iter)]
///     quxe: Vec<Instr>
/// }
///
pub struct Dummy;
// pub fn parse_attr(attr: &syn::Attribute) -> Option<Vec<syn::NestedMeta>> {

//     None
// }

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use syn::parse2;
    use syn::{parse_quote, Attribute};

    #[test]
    fn range_start() {
        let attr: Attribute = parse_quote! { #[snom(range(start = 1))] };
        // assert!(is_structnom_attr(&attr));
        let snom_arg = parse2::<SnomArg>(attr.tts).unwrap();
    }

    #[test]
    fn range_end() {
        let attr: Attribute = parse_quote! { #[snom(range(end = 1))] };
        let snom_arg = parse2::<SnomArg>(attr.tts).unwrap();
    }

    #[test]
    fn range_skip() {
        let attr_1: Attribute = parse_quote! { #[snom(range(skip))] };
        let snom_arg = parse2::<SnomArg>(attr_1.tts).unwrap();

        let attr_2: Attribute = parse_quote! { #[snom(range(skip = 5))] };
        let snom_arg = parse2::<SnomArg>(attr_2.tts).unwrap();
    }

    #[test]
    fn val() {
        let attr: Attribute = parse_quote! { #[snom(val = 10)] };
        let snom_arg = parse2::<SnomArg>(attr.tts).unwrap();
    }

    #[test]
    fn values() {
        let attr: Attribute = parse_quote! { #[snom(values(0x01, 0x02, 0x03))] };
        let snom_arg = parse2::<SnomArg>(attr.tts).unwrap();
    }

    #[test]
    fn skip() {
        let attr: Attribute = parse_quote! { #[snom(skip)] };
        let snom_arg = parse2::<SnomArg>(attr.tts).unwrap();
    }

    #[test]
    fn debug() {
        let attr: Attribute = parse_quote! { #[snom(debug)] };
        let snom_arg = parse2::<SnomArg>(attr.tts).unwrap();

        let attr: Attribute = parse_quote! { #[snom(debug = "0x{:x?}")] };
        let snom_arg = parse2::<SnomArg>(attr.tts).unwrap();
    }

    #[test]
    fn iter() {
        let attr: Attribute = parse_quote! { #[snom(iter)] };
        let snom_arg = parse2::<SnomArg>(attr.tts).unwrap();
    }

    #[test]
    fn parser() {
        let attr: Attribute = parse_quote! { #[snom(parser = "crate::leb_u32")] };
        let snom_arg = parse2::<SnomArg>(attr.tts).unwrap();
    }
}
