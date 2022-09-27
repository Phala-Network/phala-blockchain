use std::convert::TryFrom;

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse::Parse, punctuated::Punctuated, spanned::Spanned, Result, Token};

use ink_lang_ir::{HexLiteral as _, ImplItem, Selector};

#[derive(Debug, PartialEq, Eq)]
struct MetaNameValue {
    name: syn::Ident,
    eq_token: syn::token::Eq,
    value: syn::Path,
}

impl Parse for MetaNameValue {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        Ok(Self {
            name: input.parse()?,
            eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

pub fn patch(input: TokenStream2, config: TokenStream2) -> TokenStream2 {
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
                                let crate_pink_extenson = crate::find_crate_name("pink-extension")?;
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

    let mut inner = None;

    let attrs = std::mem::take(&mut module.attrs);
    for attr in attrs.into_iter() {
        if !attr.path.is_ident("pink") {
            module.attrs.push(attr);
            continue;
        }

        let args: Punctuated<MetaNameValue, Token![,]> =
            attr.parse_args_with(Punctuated::parse_terminated)?;
        for arg in args.into_iter() {
            if arg.name.to_string() == "inner" {
                inner = Some(arg.value);
            }
        }
    }

    let crate_ink_lang = crate::find_crate_name("ink_lang")?;
    let inner_contract = match inner {
        Some(inner) => quote! {
            #inner
        },
        None => quote! {
            #crate_ink_lang::contract
        },
    };
    Ok(quote! {
        #[#inner_contract(#config)]
        #module
    })
}
