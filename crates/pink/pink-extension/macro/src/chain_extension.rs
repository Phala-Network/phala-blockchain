use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse::Parse, Result, Type};
use unzip3::Unzip3 as _;

use ink_lang_ir::ChainExtension;

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

pub(crate) fn patch(input: TokenStream2) -> TokenStream2 {
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
            !matches!(i, &syn::TraitItem::Type(_))
        });

        item_trait.items.push(syn::parse_quote! {
            type Error;
        });

        for item in item_trait.items.iter_mut() {
            if let syn::TraitItem::Method(item_method) = item {
                item_method.attrs.clear();

                // Turn &[u8] into Cow<[u8]>
                for input in item_method.sig.inputs.iter_mut() {
                    match input {
                        syn::FnArg::Receiver(_) => (),
                        syn::FnArg::Typed(arg) => {
                            if let Type::Reference(tp) = *arg.ty.clone() {
                                let inner_type = tp.elem.clone();
                                arg.ty = syn::parse_quote! {
                                    Cow<#inner_type>
                                };
                            }
                        }
                    }
                }

                item_method.sig.inputs.insert(
                    0,
                    syn::parse_quote! {
                        &self
                    },
                );
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

    let extension = ChainExtension::new(Default::default(), input.clone())?;
    let id_pairs: Vec<_> = {
        extension
            .iter_methods()
            .map(|m| {
                let name = m.ident().to_string();
                let id = m.id().into_u32();
                let args: Vec<_> = m
                    .inputs()
                    .enumerate()
                    .map(|(i, _)| Ident::new(&format!("arg_{i}"), Span::call_site()))
                    .collect();
                (name, id, args)
            })
            .collect()
    };

    // Extract all function ids to a sub module
    let func_ids = {
        let mut mod_item: syn::ItemMod = syn::parse_quote! {
            pub mod func_ids {}
        };
        for (name, id, _) in id_pairs.iter() {
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
        let (names, ids, args): (Vec<_>, Vec<_>, Vec<_>) = id_pairs
            .into_iter()
            .map(|(name, id, args)| {
                let name = Ident::new(&name, Span::call_site());
                let id = Literal::u32_unsuffixed(id);
                (name, id, args)
            })
            .unzip3();
        syn::parse_quote! {
            #[macro_export]
            macro_rules! dispatch_ext_call {
                ($func_id: expr, $handler: expr, $env: expr) => {
                    match $func_id {
                        #(
                            #ids => {
                                let (#(#args),*) = $env.read_as_unbounded($env.in_len())?;
                                let output = $handler.#names(#(#args),*)?;
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

    // Mock helper functions
    let mock_helpers = {
        let mut mod_item: syn::ItemMod = syn::parse_quote! {
            pub mod mock {
                use super::*;
                use super::test::MockExtension;
            }
        };
        let mut reg_expressions: Vec<TokenStream2> = Default::default();
        for m in extension.iter_methods() {
            let name = m.ident().to_string();
            let fname = "mock_".to_owned() + &name;
            let fname = Ident::new(&fname, Span::call_site());
            let origin_fname = Ident::new(&name, Span::call_site());
            let id = Literal::u32_unsuffixed(m.id().into_u32());
            let input_types: Vec<Type> = m.inputs().map(|arg| (*arg.ty).clone()).collect();
            let input_types_cow: Vec<Type> = input_types
                .iter()
                .map(|arg| match arg.clone() {
                    Type::Reference(tp) => {
                        let inner = &tp.elem;
                        syn::parse_quote! { Cow<#inner> }
                    }
                    tp => tp,
                })
                .collect();
            let input_args: Vec<_> = input_types
                .iter()
                .enumerate()
                .map(|(i, _)| Ident::new(&format!("arg_{i}"), Span::call_site()))
                .collect();
            let input_args_asref: Vec<TokenStream2> = input_types
                .iter()
                .enumerate()
                .map(|(i, tp)| {
                    let name = Ident::new(&format!("arg_{i}"), Span::call_site());
                    match tp {
                        Type::Reference(_) => {
                            syn::parse_quote! {
                                #name.as_ref()
                            }
                        }
                        _ => syn::parse_quote! {
                            #name
                        },
                    }
                })
                .collect();
            let output = m.sig().output.clone();
            mod_item
            .content
            .as_mut()
            .unwrap()
            .1
            .push(syn::parse_quote! {
                pub fn #fname(mut call: impl FnMut(#(#input_types),*) #output + 'static) {
                    ink_env::test::register_chain_extension(
                        MockExtension::<_, _, _, #id>::new(
                            move |(#(#input_args),*): (#(#input_types_cow),*)| call(#(#input_args_asref),*)
                        ),
                    );
                }
            });
            reg_expressions.push(syn::parse_quote! {
            ink_env::test::register_chain_extension(
                MockExtension::<_, _, _, #id>::new(
                    move |(#(#input_args),*): (#(#input_types_cow),*)| ext_impl.#origin_fname(#(#input_args),*).unwrap()
                ),
            );
        });
        }

        let backend_trait_ident = &backend_trait.ident;
        mod_item
        .content
        .as_mut()
        .unwrap()
        .1
        .push(syn::parse_quote! {
            pub fn mock_all_with<E: core::fmt::Debug, I: #backend_trait_ident<Error=E>>(ext_impl: &'static I) {
                #(#reg_expressions)*
            }
        });
        mod_item
    };

    let crate_ink_lang = crate::find_crate_name("ink_lang")?;
    Ok(quote! {
        #[#crate_ink_lang::chain_extension]
        #input

        #backend_trait

        #func_ids

        #dispatcher

        #[cfg(feature = "std")]
        #mock_helpers
    })
}
