use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_crate::{crate_name, FoundCrate};
use syn::{parse_macro_input, Result};

mod contract;
mod chain_extension;

/// A drop-in replacement for `ink_lang::contract` with pink-specific feature extensions.
///
/// # pink-specific features
/// - `#[pink(on_block_end)]`
///   Marks a function as being called on each phala block has been dispatched.
#[proc_macro_attribute]
pub fn contract(arg: TokenStream, input: TokenStream) -> TokenStream {
    let config = parse_macro_input!(arg as TokenStream2);
    let module = parse_macro_input!(input as TokenStream2);
    let module = contract::patch(module, config);
    module.into()
}

/// Internal use only.
#[proc_macro_attribute]
pub fn chain_extension(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TokenStream2);
    let output = chain_extension::patch(input);
    output.into()
}

fn find_crate_name(origin: &str) -> Result<syn::Ident> {
    use proc_macro2::Span;
    let name = match crate_name(origin) {
        Ok(FoundCrate::Itself) => syn::Ident::new("crate", Span::call_site()),
        Ok(FoundCrate::Name(alias)) => syn::Ident::new(&alias, Span::call_site()),
        Err(e) => {
            return Err(syn::Error::new(Span::call_site(), &e));
        }
    };
    Ok(name)
}
