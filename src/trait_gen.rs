use quote::quote;

use crate::Endian;
// use proc_macro2;

/// generate_structnom!(r#"
///     endian = { native, little, big }
///     streaming = { true, false, both }
///     iterating = { true, false, both }    
///     verbose-errors = { true, false
///     vector-style = {
///         length = endian_u8, fn -> Integer
///         terminal = None, &[u8]
///         included = false, true
///     }
///     string-style = {
///         IsVector,
///         length = endian_u8, fn -> Integer
///         terminal = None, &[u8]
///         included = false, true
///     }
///
///     // primitive numeric types are in the form:
///     // {type}-parser = fn -> Integer
///     // They default to endian_{type}
///     
/// "#);
///
pub struct Unit;
// TraitConfig {
//     endian: { Native, Little, Big },
//     streaming: { true, false, both },
//     iterating: { true, false, both },
//     verbose-errors = { true, false },
//     vector-style = VectorStyle {
//         length = Option<"path">,
//         terminal = Option<&[u8]>,
//         included = { false, true },
//     },
//     string-style = StringStyle {
//         IsVector,
//         Style {
//             length = Option<"path">,
//             terminal = Option<&[u8]>,
//             included = false, true
//         }
//     }
// }

// pub struct TraitGenerator {
//     fn generate() -> proc_macro2::TokenStream {
//         quote!()
//     }
// }

