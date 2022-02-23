use std::convert::TryFrom;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_crate::{crate_name, FoundCrate};
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, Result};

use ink_lang_ir::{ChainExtension, HexLiteral as _, ImplItem, Selector};

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

/// Internal use only.
#[proc_macro_attribute]
pub fn chain_extension(_: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TokenStream2);
    let output = patch_chain_extension(input);
    output.into()
}

fn patch_chain_extension(input: TokenStream2) -> TokenStream2 {
    match patch_chain_extension_or_err(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
}

fn patch_chain_extension_or_err(input: TokenStream2) -> Result<TokenStream2> {
    use proc_macro2::{Ident, Literal, Span};

    let backend_trait = {
        let mut item_trait: syn::ItemTrait = syn::parse2(input.clone())?;

        item_trait.ident = syn::Ident::new(
            &format!("{}Backend", item_trait.ident),
            item_trait.ident.span(),
        );

        item_trait.items.retain(|i| {
            if let &syn::TraitItem::Type(_) = i {
                false
            } else {
                true
            }
        });

        item_trait.items.push(syn::parse_quote! {
            type Error;
        });

        for item in item_trait.items.iter_mut() {
            if let syn::TraitItem::Method(item_method) = item {
                item_method.attrs.clear();
                item_method.sig.inputs.insert(0, syn::parse_quote! {
                    &self
                });
                item_method.sig.output = match item_method.sig.output.clone() {
                    syn::ReturnType::Type(_, tp) => {
                        syn::parse_quote! {
                            -> Result<#tp, Self::Error>
                        }
                    }
                    syn::ReturnType::Default => {
                        syn::parse_quote! {
                            -> Result<(), Self::Error>
                        }
                    }
                };
            }
        }

        item_trait
    };

    let id_pairs: Vec<_> = {
        let extension = ChainExtension::new(Default::default(), input.clone())?;
        extension
            .iter_methods()
            .map(|m| {
                let name = m.ident().to_string();
                let id = m.id().into_u32();
                (name, id)
            })
            .collect()
    };

    // Extract all function ids to a sub module
    let func_ids = {
        let mut mod_item: syn::ItemMod = syn::parse_quote! {
            pub mod func_ids {}
        };
        for (name, id) in id_pairs.iter() {
            let name = name.to_uppercase();
            let name = Ident::new(&name, Span::call_site());
            let id = Literal::u32_unsuffixed(*id);
            mod_item
                .content
                .as_mut()
                .unwrap()
                .1
                .push(syn::parse_quote! {
                    pub const #name: u32 = #id;
                });
        }
        mod_item
    };

    // Generate the dispatcher
    let dispatcher: syn::ItemMacro = {
        let (names, ids): (Vec<_>, Vec<_>) = id_pairs
            .into_iter()
            .map(|(name, id)| {
                let name = Ident::new(&name, Span::call_site());
                let id = Literal::u32_unsuffixed(id);
                (name, id)
            })
            .unzip();
        syn::parse_quote! {
            #[macro_export]
            macro_rules! dispatch_ext_call {
                ($func_id: expr, $handler: expr, $env: expr) => {
                    match $func_id {
                        #(
                            #ids => {
                                let input = $env.read_as_unbounded($env.in_len())?;
                                let output = $handler.#names(input)?;
                                let output = output.encode();
                                Some(output)
                            }
                        )*
                        _ => None,
                    }
                };
            }
        }
    };

    let crate_ink_lang = find_crate_name("ink_lang")?;
    Ok(quote! {
        #[#crate_ink_lang::chain_extension]
        #input

        #backend_trait

        #func_ids

        #dispatcher
    })
}
