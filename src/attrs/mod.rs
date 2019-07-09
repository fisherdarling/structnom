pub mod attrs;

pub use attrs::*;

use syn::Lit::{self, *};
use syn::{Error, ExprPath, Result};

fn get_expr_path(lit: &Lit) -> Result<ExprPath> {
    match lit {
        Str(lit_str) => lit_str.parse(),
        _ => {
            let message = "expected in the form \"path::to::something\"";
            Err(Error::new_spanned(lit, message))
        }
    }
}
