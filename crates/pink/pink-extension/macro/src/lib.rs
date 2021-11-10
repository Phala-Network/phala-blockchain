use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Result};

use ink_lang_ir::{HexLiteral as _, Selector};

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
                            attrs.push(syn::parse_quote! {
                                #[ink(message)]
                            });

                            let method_name = item_method.sig.ident.to_string();
                            let selector = Selector::new(method_name.as_bytes()).into_be_u32();
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
            }
        }
    }
    let export_selector = match block_end_selector {
        Some(selector) => {
            let selector = selector.hex_suffixed();
            quote! {
                #[no_mangle]
                fn pink_on_block_end_selector() {
                    let selector = #selector;
                    ink_env::return_value(Default::default(), &selector);
                }
            }
        }
        None => quote! {},
    };

    Ok(quote! {
        #export_selector

        #[ink_lang::contract(#config)]
        #module
    })
}
