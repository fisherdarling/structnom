use nom::le_u8;
use nom::take;
use nom::ErrorKind;
use nom::IResult;
use nom::{le_f32, le_f64, le_u64, le_u32};





pub trait Nom {
    fn nom(input: &[u8]) -> IResult<&[u8], Self>
    where
        Self: Sized;
}

impl Nom for u8 {
    fn nom(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, res) = le_u8(input)?;

        Ok((input, res))
    }
}

impl Nom for u32 {
    fn nom(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, res) = le_u32(input)?;

        Ok((input, res))
    }
}

impl<T: Nom> Nom for Vec<T> {
    fn nom(input: &[u8]) -> IResult<&[u8], Self> {
        parse_vec(input)
    }
}

impl<T: Nom> Nom for Option<T> {
    fn nom(input: &[u8]) -> IResult<&[u8], Self> {
        opt!(input, complete!(T::nom))
    }
}

impl Nom for String {
    fn nom(input: &[u8]) -> IResult<&[u8], Self> {
        let (input, bytes) = <Vec<u8>>::nom(input)?;

        Ok((input, String::from_utf8(bytes).unwrap()))
    }
}

pub fn parse_vec<T: Nom>(data: &[u8]) -> IResult<&[u8], Vec<T>> {
    let (input, length) = le_u8(data)?;

    // println!("Parsing vec of length {}", length);

    count!(input, Nom::nom, length as usize)
}

// // TODO: Figure out work around with :: for type parameters
// pub fn parse_functype(input: &[u8]) -> IResult<&[u8], FuncType> {
//     let (input, _) = tag!(input, &[0x60u8])?;
//     let (input, params) = parse_vec::<ValType>(input)?;
//     let (input, results) = parse_vec::<ResType>(input)?;

//     Ok((input, FuncType::new(params, results)))
// }