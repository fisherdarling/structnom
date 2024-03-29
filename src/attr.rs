//! Parsing logic for StructNom attributes.

use proc_macro2::TokenStream;
use quote::{quote, ToTokens, TokenStreamExt};
use syn::parse::{Lookahead1, Parse, ParseBuffer, ParseStream, Peek};
use syn::{
    parenthesized, punctuated::Punctuated, token::Token, Attribute, LitInt, LitStr, Meta, Token,
};
// use syn::lookahead::TokenMarker;

use syn::Result as SynResult;

#[derive(Debug, Clone, PartialEq)]
pub enum SnomArg {
    Match(MatchArg),
    Parser(ValueArg),
    Effect(EffectArg),
    None,
}

impl SnomArg {
    pub fn parse(attr: &Attribute) -> SynResult<Option<Self>> {
        // println!("SnomArg parse: {:?}", attr);

        if is_structnom_attr(attr) {
            // println!("Is Snom Attr");

            let parsed = syn::parse2::<SnomArg>(attr.tts.clone());
            // println!("Parsed Attr: {:?}", parsed);

            Ok(Some(parsed?))
        } else {
            Ok(None)
        }
    }

    pub fn match_arg(&self) -> Option<&MatchArg> {
        if let SnomArg::Match(ref a) = self {
            Some(a)
        } else {
            None
        }
    }

    pub fn value_arg(&self) -> Option<&ValueArg> {
        if let SnomArg::Parser(ref a) = self {
            Some(a)
        } else {
            None
        }
    }

    pub fn effect_arg(&self) -> Option<&EffectArg> {
        if let SnomArg::Effect(ref a) = self {
            Some(a)
        } else {
            None
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

        // println!("SnomArg Input: {:?}", input);

        let lookahead = input.lookahead1();

        // println!("SnomArg: input {:#?}", input);

        if looking_at_match(&lookahead) {
            // println!("Looking At: {}", "match");
            
            Ok(SnomArg::Match(input.parse()?))
        } else if looking_at_parser(&lookahead) {
            // println!("Looking At: {}", "parser");
            
            Ok(SnomArg::Parser(input.parse()?))
        } else if looking_at_effect(&lookahead) {
            // println!("Looking At: {}", "effect");
            
            Ok(SnomArg::Effect(input.parse()?))
        } else {
            // println!("Looking At: {}", "none");
            
            Ok(SnomArg::None)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum ValueArg {
    Parser {
        parser_token: kw::parser,
        eq_token: Token![=],
        value: syn::Path,
    },
    Bits {
        bits_token: kw::bits,
        parens_token: syn::token::Paren,
        count: LitInt,
    },
    TagBits {
        bits_token: kw::bits,
        parens_token: syn::token::Paren,
        count: LitInt,
        comma_token: Token![,],
        pattern: syn::Pat,
    },
    Skip {
        skip_token: kw::skip,
    },
    Iter {
        iter_token: kw::iter,
    },
    Switch {
        switch_token: kw::switch,
        eq_token: Token![=],
        value: syn::Path,
    },
}

impl Parse for ValueArg {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let lookahead = input.lookahead1();

        // println!("ValueArg Input: {:?}", input);

        if lookahead.peek(kw::parser) {
            Ok(ValueArg::Parser {
                parser_token: input.parse()?,
                eq_token: input.parse()?,
                value: input.parse()?,
            })
        } else if lookahead.peek(kw::skip) {
            Ok(ValueArg::Skip {
                skip_token: input.parse()?,
            })
        } else if lookahead.peek(kw::iter) {
            Ok(ValueArg::Iter {
                iter_token: input.parse()?,
            })
        } else if lookahead.peek(kw::switch) {
            Ok(ValueArg::Switch {
                switch_token: input.parse()?,
                eq_token: input.parse()?,
                value: input.parse()?,
            })
        } else if lookahead.peek(kw::bits) {
            let content;

            let bits_token: kw::bits = input.parse()?;
            let parens_token: syn::token::Paren = parenthesized!(content in input);
            let count: LitInt = content.parse()?;

            // println!("*** Lookahead ***", content.look)

            if content.peek(Token![,]) {
                let comma_token: Token![,] = content.parse()?;
                let pattern: syn::Pat = content.parse()?;

                let val = Ok(ValueArg::TagBits {
                    bits_token,
                    parens_token,
                    count,
                    comma_token,
                    pattern,
                });

                // println!("TagBits: {:?}", val);

                val
            } else {
                let val = Ok(ValueArg::Bits {
                    bits_token,
                    parens_token,
                    count,
                });

                // println!("=== Bits ===: {:?}", val);

                val
            }
        } else {
            Err(lookahead.error())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EffectArg {
    Tag {
        tag_token: kw::tag,
        paren_token: syn::token::Paren,
        value: TagEither,
    },
    Take {
        take_token: kw::take,
        paren_token: syn::token::Paren,
        value: LitInt,
    },
    Debug {
        debug_token: kw::debug,
        eq_token: Option<Token![=]>,
        value: Option<LitStr>,
    },
    Call {
        call_token: kw::call,
        paren_token: syn::token::Paren,
        value: syn::Path,
    }
}

impl Parse for EffectArg {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let lookahead = input.lookahead1();

        // println!("+++ EffectArg Input: {}", input);

        if lookahead.peek(kw::tag) {
            let content;
            Ok(EffectArg::Tag {
                tag_token: input.parse()?,
                paren_token: parenthesized!(content in input),
                value: content.parse()?,
            })
        } else if lookahead.peek(kw::debug) {
            Ok(EffectArg::Debug {
                debug_token: input.parse()?,
                eq_token: input.parse()?,
                value: input.parse()?,
            })
        } else if lookahead.peek(kw::call) {
            let content;
            Ok(EffectArg::Call {
                call_token: input.parse()?,
                paren_token: parenthesized!(content in input),
                value: content.parse()?,
            })
        } else if lookahead.peek(kw::take) {
            let content;
            Ok(EffectArg::Take {
                take_token: input.parse()?,
                paren_token: parenthesized!(content in input),
                value: content.parse()?,
            })
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for EffectArg {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let repr = match self {
            EffectArg::Tag { value, .. } => match value {
                TagEither::Slice(ident) => quote! { tag!(#ident) },
                TagEither::Values(vals) => quote! { tag!(&[#(#vals),*]) },
            },
            EffectArg::Debug { value, .. } => {
                quote! { compile_error!("Debug is not yet implemented.") }
            }
            EffectArg::Call { value, .. } => {
                quote! { call!(#value) }
            }
            EffectArg::Take { value, .. } => {
                quote!(take!(#value))
            }
        };

        tokens.extend(repr);
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

pub fn looking_at_match(lookahead: &Lookahead1) -> bool {
    lookahead.peek(kw::range) || lookahead.peek(kw::val) || lookahead.peek(kw::values)
}

pub fn looking_at_parser(lookahead: &Lookahead1) -> bool {
    lookahead.peek(kw::parser)
        || lookahead.peek(kw::skip)
        || lookahead.peek(kw::iter)
        || lookahead.peek(kw::switch)
        || lookahead.peek(kw::bits)
}

pub fn looking_at_effect(lookahead: &Lookahead1) -> bool {
    lookahead.peek(kw::debug) || lookahead.peek(kw::tag) || lookahead.peek(kw::call) || lookahead.peek(kw::take)
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
    custom_keyword!(switch);
    custom_keyword!(parser);
    custom_keyword!(iter);
    custom_keyword!(bits);
    custom_keyword!(tag);
    // custom_keyword!(complete);

    custom_keyword!(call);
    custom_keyword!(take);
    custom_keyword!(debug);
}

#[derive(Debug, Clone, PartialEq)]
pub enum TagEither {
    Slice(syn::Ident),
    Values(Punctuated<LitInt, Token![,]>),
}

impl Parse for TagEither {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let lookahead = input.lookahead1();

        if lookahead.peek(syn::Ident) {
            Ok(TagEither::Slice(input.parse()?))
        } else if lookahead.peek(LitInt) {
            Ok(TagEither::Values(input.parse_terminated(LitInt::parse)?))
        } else {
            Err(lookahead.error())
        }
    }
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
