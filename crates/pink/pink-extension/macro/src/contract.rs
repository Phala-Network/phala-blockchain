use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse::Parse, punctuated::Punctuated, Result, Token};

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
    let mut inner = None;

    let attrs = std::mem::take(&mut module.attrs);
    for attr in attrs.into_iter() {
        if !attr.path().is_ident("pink") {
            module.attrs.push(attr);
            continue;
        }

        let args: Punctuated<MetaNameValue, Token![,]> =
            attr.parse_args_with(Punctuated::parse_terminated)?;
        for arg in args.into_iter() {
            if arg.name == "inner" {
                inner = Some(arg.value);
            }
        }
    }

    let crate_ink_lang = crate::find_crate_name("ink")?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works_with_inner() {
        let config: TokenStream2 = syn::parse_quote!(env = PinkEnvironment);
        let body: TokenStream2 = syn::parse_quote! {
            #[ink::contract]
            #[pink(inner = ink::contract)]
            mod test {
                #[ink(storage)]
                pub struct Test {
                    flag: String,
                }

                impl Test {
                    #[ink(constructor)]
                    pub fn new() -> Self {
                        Self {
                            flag: String::new(),
                        }
                    }

                    #[ink(message)]
                    pub fn flag(&self) -> String {
                        self.flag.clone()
                    }
                }
            }
        };
        let output = patch(body, config);
        insta::assert_snapshot!(rustfmt_snippet::rustfmt_token_stream(&output).unwrap());
    }

    #[test]
    fn it_works_without_inner() {
        let config: TokenStream2 = syn::parse_quote!(env = PinkEnvironment);
        let body: TokenStream2 = syn::parse_quote! {
            #[ink::contract]
            mod test {
                #[ink(storage)]
                pub struct Test {
                    flag: String,
                }
                impl Test {
                    #[ink(constructor)]
                    pub fn new() -> Self {
                        Self {
                            flag: String::new(),
                        }
                    }
                    #[ink(message)]
                    pub fn flag(&self) -> String {
                        self.flag.clone()
                    }
                }
            }
        };
        let output = patch(body, config);
        insta::assert_snapshot!(rustfmt_snippet::rustfmt_token_stream(&output).unwrap());
    }

    #[test]
    fn test_invalid_config() {
        let config: TokenStream2 = syn::parse_quote!(env = PinkEnvironment);
        let body: TokenStream2 = syn::parse_quote! {
            #[pink(inner = ink::contract;)]
            mod test {
                #[ink(storage)]
                pub struct Test {
                    flag: String,
                }
                impl Test {
                    #[ink(constructor)]
                    pub fn new() -> Self {
                        Self {
                            flag: String::new(),
                        }
                    }
                    #[ink(message)]
                    pub fn flag(&self) -> String {
                        self.flag.clone()
                    }
                }
            }
        };
        let output = patch(body, config);
        assert_eq!(
            rustfmt_snippet::rustfmt_token_stream(&output).unwrap(),
            "::core::compile_error! { \"expected `,`\" }\n"
        );
    }
}
