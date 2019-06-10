#![feature(specialization)]

use structnom_derive::*;

// #[macro_use]
// extern crate nom;

use nom::*;


generate_structnom!(little);

// static SOME_SLICE: &[u8] = &[1, 2, 3, 4];

#[derive(StructNom)]
#[snom(switch = le_u8)]
pub enum Instr {
    #[snom(range(start = 1))]
    Nop, // 1
    If,  // 2
    // #[snom(parser = crate::my_parser)]
    Add, // 3
    #[snom(range(skip))]
    Sub, // 5
    #[snom(range(skip = 3))]
    Skip3, // 9
    #[snom(range(end = 10))]
    Equal, // 10
    #[snom(val = 0x0F)]
    Another, // 15
}

#[derive(StructNom)]
pub struct Example<T: StructNom> {
    // #[snom(debug = "0x{:x?}")]
    #[snom(parser = nom::le_u32)]
    foo: u32,
    #[snom(skip)]
    bar: Vec<T>,
    quxe: Vec<Instr>
}


// #[derive(Debug, StructNom)]
// pub struct MyStruct {
//     // #[snom(tag(0x03, 0x04, 0x05, 0x06))]
//     first: u32,
//     last: u64,
// }

fn main() {

}