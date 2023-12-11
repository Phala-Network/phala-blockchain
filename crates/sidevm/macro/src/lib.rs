use proc_macro::TokenStream;
use syn::Result;

mod macro_main;
mod macro_ocall;
#[cfg(test)]
mod tests;

#[proc_macro_attribute]
pub fn ocall(_: TokenStream, input: TokenStream) -> TokenStream {
    macro_ocall::patch(syn::parse_macro_input!(input)).into()
}

/// Mark the entry point of the Sidevm module.
#[proc_macro_attribute]
pub fn main(_: TokenStream, input: TokenStream) -> TokenStream {
    macro_main::patch(syn::parse_macro_input!(input)).into()
}

#[cfg(not(test))]
fn find_crate_name(origin: &str) -> Result<syn::Ident> {
    use proc_macro2::Span;
    use proc_macro_crate::{crate_name, FoundCrate};
    let name = match crate_name(origin) {
        Ok(FoundCrate::Itself) => syn::Ident::new("crate", Span::call_site()),
        Ok(FoundCrate::Name(alias)) => syn::Ident::new(&alias, Span::call_site()),
        Err(e) => {
            return Err(syn::Error::new(Span::call_site(), &e));
        }
    };
    Ok(name)
}

#[cfg(test)]
fn find_crate_name(origin: &str) -> Result<syn::Ident> {
    use heck::ToSnakeCase;
    use proc_macro2::Span;
    Ok(syn::Ident::new(&origin.to_snake_case(), Span::call_site()))
}
