#![feature(custom_attribute, specialization, trivial_bounds)]

#[macro_use]
extern crate nom;

#[macro_use]
extern crate structnom_derive;

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

pub trait MyMarker {}

impl<T: StructNom + MyMarker> StructNom for Vec<T> {
    fn nom(input: &[u8]) -> nom::IResult<&[u8], Self> {
        let (input, res) = T::nom(input)?;

        Ok((input, vec![res]))
    }
}

#[derive(Debug, StructNom)]
pub struct MyStruct;

impl MyMarker for Vec<u8> {}

#[derive(Debug, StructNom)]
pub struct Expression(Vec<u8>);

fn main() {}
