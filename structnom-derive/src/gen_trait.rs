use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::Ident;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Endian {
    Little,
    Big,
}

impl Endian {
    pub fn prefix(&self, func: &str) -> Ident {
        let prefix = match self {
            Endian::Little => "le",
            Endian::Big => "be",
        };

        let concat = format!("{}_{}", prefix, func);

        Ident::new(&concat, proc_macro2::Span::call_site())
    }
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

macro_rules! numeric_impl {
    ($name:ident, $ty1:ty, $ty2:ty) => {
        pub fn $name(endian: Endian) -> proc_macro2::TokenStream {
            let func_a = endian.prefix(&stringify!($ty1));
            let func_b = endian.prefix(&stringify!($ty2));

            let expanded = quote! {
                impl StructNom for $ty1 {
                    fn nom(input: &[u8]) -> nom::IResult<&[u8], Self> {
                        let (input, res) = nom::#func_a(input)?;

                        Ok((input, res))
                    }
                }

                impl StructNom for $ty2 {
                    fn nom(input: &[u8]) -> nom::IResult<&[u8], Self> {
                        let (input, res) = nom::#func_b(input)?;

                        Ok((input, res))
                    }
                }
            };

            expanded
        }
    };
}

numeric_impl!(gen_byte_impl, u8, i8);
numeric_impl!(gen_hword_impl, u16, i16);
numeric_impl!(gen_word_impl, u32, i32);
numeric_impl!(gen_long_impl, u64, i64);
numeric_impl!(gen_float_impl, f32, f64);

pub fn gen_vec_impl(endian: Endian) -> proc_macro2::TokenStream {
    let func = endian.prefix("u8");

    let expanded = quote! {
        impl<T: StructNom> StructNom for Vec<T> {
            default fn nom(input: &[u8]) -> nom::IResult<&[u8], Self> {
                let (input, length) = nom::#func(input)?;
                let (input, res) = count!(input, T::nom, length as usize)?;

                Ok((input, res))
            }
        }
    };

    expanded
}

pub fn gen_option_impl() -> proc_macro2::TokenStream {
    let expanded = quote! {
        impl<T: StructNom> StructNom for Option<T> {
            default fn nom(input: &[u8]) -> nom::IResult<&[u8], Self> {
                let (input, res) = opt!(input, T::nom)?;

                Ok((input, res))
            }
        }
    };

    expanded
}
