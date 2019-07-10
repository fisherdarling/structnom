use crate::attrs::Container;

use syn::DataStruct;

pub struct StructGenerator {
    container: Container,
    data: DataStruct,
}

impl StructGenerator {
    pub fn new(container: Container, data: DataStruct) -> StructGenerator {
        Self {
            container,
            data,
        }
    }

    pub fn generate(&self) -> proc_macro2::TokenStream {
        unimplemented!()
    }
}