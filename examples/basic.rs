#![feature(custom_attribute)]

#[macro_use]
extern crate nom;

#[macro_use]
extern crate structnom;

#[macro_use]
extern crate structnom_derive;

use structnom_derive::generate_nom_trait;

use structnom::Nom;

use nom::le_u32;

use nom::IResult;

#[derive(Debug, Nom)]
pub struct Signature;

#[derive(Debug, Nom)]
pub struct Expression;

#[derive(Nom)]
pub struct UnnamedField(Expression, Signature);

#[derive(Nom)]
pub struct NamedField {
    expr: Expression,
    sig: Signature,
}

#[derive(Debug, Nom)]
pub struct FieldAttr {
    expr: Expression,
    #[tag(0x10, 0x11)]
    sig: Signature,
}

use nom::le_u8;

named!(
    parse_bytes,
    take!(4)
);

#[derive(Debug, Nom)]
#[switch(le_u8)]
pub enum MyEnum {
    #[byte(0x11)]
    First(Expression),
    #[byte(0x01)]
    Second,
    #[byte(0x05)]
    Named {
        expr: Expression,
        sig: Signature,
    }
}


generate_nom_trait!(big);

fn main() {
    let input: &[u8] = &[0x11];

    let (input, res) = MyEnum::nom(input).unwrap();

    println!("{:?}", res);
}
