#[macro_use]
extern crate structnom_derive;

pub use structnom_derive::*;

#[macro_use]
extern crate nom;

use nom::*;

fn nom(input: &[u8]) -> nom::IResult<&[u8], ((), ())> {
    let res = 
        do_parse!(
            input,
            field_0: do_parse!(
                        tag!(&[0x60]) >> 
                        value: value!(()) >> 
                        (value)) >> 
            field_1: value!(()) >> 
            ((field_0, field_1))
        );
    res
}
