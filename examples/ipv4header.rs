// #![feature(specialization)]

// use nom::*;
// use structnom_derive::*;

// generate_structnom!(big);


// #[derive(Debug, StructNom)]
// pub struct Ipv4Header {
//     version_and_length: u8
//     #[snom(bits(6))]
//     dscp: u8,
//     #[snom(bits(2))]
//     ecn: u8,
//     total_len: u16,
//     id: u16,
//     #[snom(bits(3))]
//     flags: u8,
//     #[snom(bits(13))]
//     frag_offset: u16,
//     ttl: u8,
//     proto: u8,
//     checksum: u16,
//     source_addr: u32,
//     dest_addr: u32,
// }

// fn main() {
//     let data: &[u8] = &[0x45, 0xFF]; //, 0x00, 0x44, 0xad, 0x0b, 0x00, 0x00, 0x40, 0x11, 0x72, 0x72, 0xac, 0x14, 0x02, 0xfd, 0xac, 0x14, 0x00, 0x06];
//     // let slice = nom::types::CompleteByteSlice::from(data);

//     // let (data, (version, ihl)) = do_parse!(data,
//     //     version: bits!(take_bits!(u8, 4)) >>
//     //     ihl: bits!(take_bits!(u8, 4)) >>
//     //     ((version, ihl))
//     // ).unwrap();

//     let (data, version) = bits!(data, take_bits!(u8, 4)).unwrap();
//     println!("Data: {:?}", data);
//     let (data, ihl) = bits!(data, take_bits!(u8, 4)).unwrap();
//     println!("Data: {:?}", data);
//     println!("{}, {}", version, ihl);

//     // let header = Ipv4Header::nom(&slice).unwrap();

//     // println!("Parsed Ipv4Header: {:?}", header);
// }

fn main() {}