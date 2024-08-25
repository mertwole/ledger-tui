#[macro_use]
extern crate quote;
extern crate proc_macro;
extern crate syn;

#[proc_macro_derive(InputMapping)]
pub fn derive_mapping(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    proc_macro::TokenStream::new()
}
