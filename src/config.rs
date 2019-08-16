use syn::{ExprPath, LitInt};

use crate::Endian;

#[derive(Default, Debug, Clone)]
pub struct TraitConfig {
    endian: Endian,
    streaming: bool,
    iterating: bool,
    vector_style: VectorStyle,
    string_style: Option<StringStyle>,
    // verbose
}

#[derive(Default, Debug, Clone)]
pub struct VectorStyle {
    length: Option<ExprPath>,
    terminal: Option<Vec<u8>>,
}

#[derive(Default, Debug, Clone)]
pub struct StringStyle {
    length: Option<ExprPath>,
    terminal: Option<Vec<LitInt>>,
}





// TraitConfig {
//     endian: { Native, Little, Big },
//     streaming: { true, false, },
//     iterating: { true, false,  },
//     verbose-errors = { true, false },
//     vector-style = VectorStyle {
//         length = Option<"path">,
//         terminal = Option<&[u8]>,
//         terminal_included = { false, true },
//     },
//     string-style = StringStyle {
//         IsVector,
//         Style {
//             length = Option<"path">,
//             terminal = Option<&[u8]>,
//             terminal_included = false, true
//         }
//     }
// }