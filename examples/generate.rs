#![feature(custom_attribute, specialization, trivial_bounds)]

#[macro_use]
extern crate nom;

#[macro_use]
extern crate structnom_derive;

use nom::le_u8;
use structnom_derive::generate_structnom;

// use structnom::StructNom;

generate_structnom!(big);

// impl StructNom for u8 {
//     // fn nom(input: &[u8]) -> nom::IResult<&[u8], Self> {
//     //     let res = nom::le_u8(input)?;

//     //     res
//     // }
// }

// impl StructNom for Vec<u8>  {
//     fn nom(input: &[u8]) -> nom::IResult<&[u8], Self> {
//         let (input, res) = u8::nom(input)?;

//         Ok((input, vec![res]))
//     }
// }

// #[derive(Debug, StructNom)]
// #[switch(le_u8)]
// pub enum MyEnum {
//     #[byte(0x40)]
//     First { field: u8 }
// }

#[derive(Debug, StructNom)]
pub struct Named {
    first: u8,
    another: u8,
}

#[derive(Debug, StructNom)]
#[switch(le_u8)]
pub enum SomeEnum {
    #[range_start(0x00)]
    First, // 0
    Second, // 1
    #[range_skip] // Skip 2
    Third, // 3
    Fourth, // 4
    #[range_end(0x05)]
    Fifth, // 5
}

// pub trait MyMarker {}

// impl<T: StructNom + MyMarker> StructNom for Vec<T> {
//     fn nom(input: &[u8]) -> nom::IResult<&[u8], Self> {
//         let (input, res) = T::nom(input)?;

//         Ok((input, vec![res]))
//     }
// }

// #[derive(Debug, StructNom)]
// pub struct MyStruct;

// impl MyMarker for Vec<u8> {}

#[derive(Debug, StructNom)]
pub struct Expression(Vec<u8>);

fn main() {
    // let input: &[u8] = &[0x41, 0x55, 0x61];

    // let (rest, value) = Unit::nom(input).unwrap();

    // log::debug!("Value: {:?}", value);
}
