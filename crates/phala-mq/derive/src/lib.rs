use proc_macro::{self, TokenStream};
use proc_macro2::{Ident, Span};
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Error};

#[proc_macro_derive(DummyMessageHashing)]
pub fn derive_dummy(_input: TokenStream) -> TokenStream {
    TokenStream::default()
}

#[proc_macro_derive(MessageHashing)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input: DeriveInput = parse_macro_input!(input);

    let name = &input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let where_clause = if where_clause.is_some() {
        quote! {
            #where_clause, #name #ty_generics: _parity_scale_codec::Encode
        }
    } else {
        quote! {
            where #name #ty_generics: _parity_scale_codec::Encode
        }
    };

    let output = quote! {
        impl #impl_generics _phala_mq::traits::MessageHashing for #name #ty_generics #where_clause {
            fn hash(&self) -> _phala_mq::MqHash {
                _phala_mq::hash(&_parity_scale_codec::Encode::encode(&self))
            }
        }
    };
    wrap_with_dummy_const(output)
}

fn include_crates() -> proc_macro2::TokenStream {
    let extern_scale = match crate_name("parity-scale-codec") {
        Ok(FoundCrate::Itself) => quote!(
            extern crate parity_scale_codec as _parity_scale_codec;
        ),
        Ok(FoundCrate::Name(parity_codec_crate)) => {
            let ident = Ident::new(&parity_codec_crate, Span::call_site());
            quote!( extern crate #ident as _parity_scale_codec; )
        }
        Err(e) => Error::new(Span::call_site(), &e).to_compile_error(),
    };
    let extern_mq = match crate_name("phala-mq") {
        Ok(FoundCrate::Itself) => quote!(
            extern crate phala_mq as _phala_mq;
        ),
        Ok(FoundCrate::Name(parity_codec_crate)) => {
            let ident = Ident::new(&parity_codec_crate, Span::call_site());
            quote!( extern crate #ident as _phala_mq; )
        }
        Err(e) => Error::new(Span::call_site(), &e).to_compile_error(),
    };
    quote! {
        #extern_scale
        #extern_mq
    }
}

fn wrap_with_dummy_const(impl_block: proc_macro2::TokenStream) -> proc_macro::TokenStream {
    let extern_crates = include_crates();
    let generated = quote! {
        const _: () = {
            #extern_crates
            #impl_block
        };
    };
    generated.into()
}
