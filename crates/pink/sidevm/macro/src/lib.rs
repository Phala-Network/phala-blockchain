use proc_macro::TokenStream;

#[cfg(test)]
mod tests;
mod ocall;

#[proc_macro_attribute]
pub fn ocall(_: TokenStream, input: TokenStream) -> TokenStream {
    ocall::patch(syn::parse_macro_input!(input)).into()
}
