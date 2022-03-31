use proc_macro::TokenStream;

mod ocall;

#[proc_macro_attribute]
pub fn ocall(_: TokenStream, input: TokenStream) -> TokenStream {
    ocall::patch(syn::parse_macro_input!(input)).into()
}
