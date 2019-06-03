#[macro_use]
pub extern crate structnom_derive;

#[macro_use]
extern crate nom;

mod nom_trait;

pub use nom_trait::Nom;
// pub use structnom_derive::nom_derive;