use structnom::StructNom;

use nom::number::complete::{le_u32, le_u8};
use nom::{do_parse, IResult};
use nom::error::VerboseError;

#[derive(StructNom)]
pub struct BitTaker {
    #[snom(call = "a::func::to::call")]
    first: u8,
    // #[snom(bits(4, 0x0A))]
    // second: u32,
}

// impl StructNom for BitTaker {
//     fn parse_slice(input: &[u8]) -> IResult<&[u8], Self> {
//         let (rest, first) = le_u8(input)?;
//         let value = BitTaker { first };
//         Ok((rest, value))
//     }
// }

// fn parse_taker(input: &[u8]) -> IResult<&[u8], BitTaker> {
//     do_parse!(
//         input,
//         first: le_u8 >>
//         // second: le_u32 >>
//         (BitTaker { first })
//     )
// }

// static SOME_SLICE: &[u8] = &[1, 2, 3, 4];

// #[derive(StructNom)]
// pub struct Example<T: StructNom> {
//     // #[snom(debug = "0x{:x?}")]
//     #[snom(parser = nom::le_u32)]
//     foo: u32,
//     #[snom(skip)]
//     bar: Vec<T>,
//     quxe: Vec<Instr>
// }

// #[derive(StructNom)]

// #[derive(StructNom)]
// #[snom(parser = "crate::mystruct_parser")]
// pub struct MyStruct {
//     #[snom(parser = "crate::memes")]
//     #[snom(skip)]
//     first: u32,
//     #[snom(take(4))]
//     last: u64,
//     #[snom(bits(4, 0x03))]
//     another: u32,
// }

// #[derive(StructNom)]
// #[snom(switch = "le_u8")]
// pub enum Instr {
//     // #[snom(range(start = 1))]
//     // Nop, // 1
//     // If, // 2
//     #[snom(call = "le_u32")]
//     #[snom(skip)]
//     Add, // 3
//     #[snom(range(skip))]
//     Sub, // 5
//     #[snom(range(skip = 3))]
//     Skip3, // 9
//     #[snom(range(end = 10))]
//     Equal, // 10
//     // #[snom(val = 0x0F)]
//     // Another, // 15
// }

fn main() {}
