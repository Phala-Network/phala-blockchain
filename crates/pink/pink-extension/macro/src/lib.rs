use std::convert::TryFrom;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{Result, parse_macro_input, spanned::Spanned};

use ink_lang_ir::{HexLiteral as _, ImplItem, Selector};

/// A drop-in replacement for `ink_lang::contract` with pink-specific feature extensions.
///
/// # pink-specific features
/// - `#[pink(on_block_end)]`
///   Marks a function as being called on each phala block has been dispatched.
#[proc_macro_attribute]
pub fn contract(arg: TokenStream, input: TokenStream) -> TokenStream {
    let config = parse_macro_input!(arg as TokenStream2);
    let module = parse_macro_input!(input as TokenStream2);
    let module = patch(module, config);
    module.into()
}

fn patch(input: TokenStream2, config: TokenStream2) -> TokenStream2 {
    match patch_or_err(input, config) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
}

fn patch_or_err(input: TokenStream2, config: TokenStream2) -> Result<TokenStream2> {
    let mut module: syn::ItemMod = syn::parse2(input)?;
    let mut block_end_selector: Option<u32> = None;

    if let Some((_, items)) = &mut module.content {
        for item in items.iter_mut() {
            if let syn::Item::Impl(item_impl) = item {
                for item in item_impl.items.iter_mut() {
                    if let syn::ImplItem::Method(item_method) = item {
                        let mut is_on_block_end = false;
                        let mut attrs = vec![];

                        // patch for #[pink(on_block_end)]
                        for attr in item_method.attrs.drain(..) {
                            if attr.path.is_ident("pink") {
                                let path = attr.parse_args::<syn::Path>()?;
                                if path.is_ident("on_block_end") {
                                    is_on_block_end = true;
                                    continue;
                                }
                            }
                            attrs.push(attr);
                        }

                        if is_on_block_end {
                            if item_method.sig.inputs.len() != 1 {
                                return Err(syn::Error::new(
                                    item_method.sig.inputs.last().unwrap().span(),
                                    "on_block_end could have no arguments",
                                ));
                            }
                            if !matches!(item_method.sig.output, syn::ReturnType::Default) {
                                return Err(syn::Error::new(
                                    item_method.sig.output.span(),
                                    "on_block_end should return no value",
                                ));
                            }
                            attrs.push(syn::parse_quote! {
                                #[ink(message)]
                            });

                            let method_name = item_method.sig.ident.to_string();
                            let selector = Selector::compute(method_name.as_bytes()).into_be_u32();
                            let literal_selector = selector.hex_suffixed();

                            attrs.push(syn::parse_quote! {
                                #[ink(selector = #literal_selector)]
                            });

                            block_end_selector = Some(selector);

                            // Ensure the caller is the runtime.
                            item_method.block.stmts.insert(
                                0,
                                syn::parse_quote! {
                                    // We use default account to indicate the message is from the runtime itself.
                                    // Assume that no one can crack the default account's private key.
                                    assert_eq!(self.env().caller(), Default::default());
                                },
                            );
                        }
                        item_method.attrs = attrs;
                    }
                }

                if let Some(selector) = block_end_selector {
                    // if on_block_end defined, instert set_on_block_end_selector in front of each constructor.
                    for item in item_impl.items.iter_mut() {
                        if let Ok(ImplItem::Constructor(_)) = ImplItem::try_from(item.clone()) {
                            if let syn::ImplItem::Method(item_method) = item {
                                let crate_pink_extenson = find_crate_name("pink-extension")?;
                                let selector = selector.hex_suffixed();
                                item_method.block.stmts.insert(
                                    0,
                                    syn::parse_quote! {
                                        #crate_pink_extenson::set_on_block_end_selector(#selector);
                                    },
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    let crate_ink_lang = find_crate_name("ink_lang")?;
    Ok(quote! {
        #[#crate_ink_lang::contract(#config)]
        #module
    })
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
