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

fn find_crate_name(origin: &str) -> syn::Result<syn::Ident> {
    use heck::ToSnakeCase;
    use proc_macro2::Span;
    use proc_macro_crate::{crate_name, FoundCrate};

    let name = match crate_name(origin) {
        Ok(FoundCrate::Itself) => syn::Ident::new("crate", Span::call_site()),
        Ok(FoundCrate::Name(alias)) => syn::Ident::new(&alias, Span::call_site()),
        Err(_) => {
            let default_name = ToSnakeCase::to_snake_case(origin);
            syn::Ident::new(&default_name, Span::call_site())
        }
    };
    Ok(name)
}
