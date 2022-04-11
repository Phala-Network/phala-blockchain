use proc_macro::TokenStream;

mod macro_main;
mod macro_ocall;
#[cfg(test)]
mod tests;

#[proc_macro_attribute]
pub fn ocall(_: TokenStream, input: TokenStream) -> TokenStream {
    macro_ocall::patch(syn::parse_macro_input!(input)).into()
}

/// Mark the entry point of the SideVM module.
#[proc_macro_attribute]
pub fn main(_: TokenStream, input: TokenStream) -> TokenStream {
    macro_main::patch(syn::parse_macro_input!(input)).into()
}
