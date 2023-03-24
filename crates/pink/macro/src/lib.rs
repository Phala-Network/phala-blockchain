use proc_macro::TokenStream;

mod macro_xcall;
#[cfg(test)]
mod tests;

#[proc_macro_attribute]
pub fn cross_call(config: TokenStream, input: TokenStream) -> TokenStream {
    macro_xcall::patch(
        syn::parse_macro_input!(config),
        syn::parse_macro_input!(input),
    )
    .into()
}
