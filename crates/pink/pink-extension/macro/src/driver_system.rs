use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{spanned::Spanned, FnArg, Result};

pub(crate) enum InterfaceType {
    System,
    Driver,
}

pub(crate) fn patch(
    input: TokenStream2,
    config: TokenStream2,
    interface: InterfaceType,
) -> TokenStream2 {
    match patch_or_err(input, config, interface) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
}

fn patch_or_err(
    input: TokenStream2,
    config: TokenStream2,
    interface: InterfaceType,
) -> Result<TokenStream2> {
    use heck::{ToLowerCamelCase, ToSnakeCase};
    let the_trait: syn::ItemTrait = syn::parse2(input)?;
    let trait_ident = &the_trait.ident;
    let trait_name = the_trait.ident.to_string();
    let trait_impl_mod = Ident::new(
        &format!("_pink_{}_impl", trait_name.to_snake_case()),
        Span::call_site(),
    );
    let impl_type = Ident::new(&format!("{trait_name}Ref"), Span::call_site());

    let crate_pink = crate::find_crate_name("pink-extension")?;
    let crate_ink_lang = crate::find_crate_name("ink")?;
    let crate_ink_env = quote!(#crate_ink_lang::env);


    let mut associated_types_t = vec![];
    let mut associated_types_v = vec![];
    let mut method_sigs = vec![];
    let mut method_forward_calls = vec![];
    let mut call_fns = vec![];

    for item in the_trait.items.iter() {
        if let syn::TraitItem::Method(method) = item {
            let method_ident = &method.sig.ident;
            associated_types_t.push({
                let assoc_t = format!(
                    "{}Output",
                    method.sig.ident.to_string().to_lower_camel_case()
                );
                Ident::new(&assoc_t, Span::call_site())
            });
            associated_types_v.push(match &method.sig.output {
                syn::ReturnType::Default => quote!(()),
                syn::ReturnType::Type(_, t) => quote!(#t),
            });
            method_sigs.push(method.sig.clone());
            method_forward_calls.push({
                let mut args = vec![];
                for p in &method.sig.inputs {
                    if let FnArg::Typed(p) = p {
                        args.push(match &*p.pat {
                            syn::Pat::Ident(id) => quote!(#id),
                            _ => {
                                return Err(syn::Error::new(p.pat.span(), "Only ident is allowed"))
                            }
                        })
                    }
                }
                quote! {
                    #method_ident(#(#args),*)
                }
            });
            call_fns.push({
                match method.sig.inputs.first() {
                    Some(FnArg::Receiver(receiver)) => {
                        if receiver.mutability.is_some() {
                            Ident::new("call_mut", Span::call_site())
                        } else {
                            Ident::new("call", Span::call_site())
                        }
                    }
                    _ => {
                        return Err(syn::Error::new(
                            method.sig.ident.span(),
                            "First arg must be self",
                        ))
                    }
                }
            });
        }
    }
    let fn_instance = match interface {
        InterfaceType::System => quote! {
            pub fn instance() -> Self {
                Self::instance_with_call_flags(CallFlags::default())
            }
            pub fn instance_with_call_flags(call_flags: CallFlags) -> Self {
                #[cfg(feature = "std")]
                if MOCK.with(|x| { x.borrow_mut().is_some() }) {
                    return Self::Mock;
                }
                Self::Instance {
                    address: #crate_pink::ext().system_contract_id(),
                    call_flags,
                }
            }
        },
        InterfaceType::Driver => {
            let driver_name = proc_macro2::Literal::string(&trait_name);
            quote! {
                pub fn instance() -> Option<Self> {
                    Self::instance_with_call_flags(CallFlags::default())
                }
                pub fn instance_with_call_flags(flags: CallFlags) -> Option<Self> {
                    #[cfg(feature = "std")]
                    if MOCK.with(|x| { x.borrow_mut().is_some() }) {
                        return Some(Self::Mock);
                    }
                    let system = #crate_pink::system::SystemRef::instance_with_call_flags(flags.clone());
                    Some(Self::Instance {
                        address: system.get_driver(#driver_name.into())?,
                        call_flags: flags,
                    })
                }
            }
        }
    };
    Ok(quote! {
        #config
        #the_trait

        pub use #trait_impl_mod::#impl_type;
        mod #trait_impl_mod {
            use super::*;
            use #crate_pink::PinkEnvironment;
            use #crate_ink_lang::{codegen::TraitCallForwarder, reflect::TraitDefinitionRegistry};
            use #crate_ink_env::call::FromAccountId;
            use #crate_ink_env::CallFlags;

            type TraitInfo = <TraitDefinitionRegistry<PinkEnvironment> as #trait_ident>::__ink_TraitInfo;
            type Forwarder = <TraitInfo as TraitCallForwarder>::Forwarder;
            pub enum #impl_type {
                Instance {
                    address: AccountId,
                    call_flags: CallFlags,
                },
                #[cfg(feature = "std")]
                Mock,
            }

            #[cfg(feature = "std")]
            thread_local! {
                static MOCK: core::cell::RefCell<Option<(AccountId, Box<
                        dyn #trait_ident<
                            Env = PinkEnvironment,
                            __ink_TraitInfo = TraitInfo,
                            #(#associated_types_t = #associated_types_v,)*
                        >,
                    >)>,
                > = Default::default();
            }

            impl #impl_type {
                #[cfg(feature = "std")]
                pub fn mock_with(
                    contract: impl #trait_ident<
                        Env = PinkEnvironment,
                        __ink_TraitInfo = TraitInfo,
                        #(#associated_types_t = #associated_types_v,)*
                    > + 'static,
                ) {
                    MOCK.with(|x| {
                        let callee = #crate_ink_env::test::callee::<PinkEnvironment>();
                        *x.borrow_mut() = Some((callee, Box::new(contract)));
                    });
                }

                pub fn set_call_flags(&mut self, flags: CallFlags) {
                    if let Self::Instance { call_flags, .. } = self {
                        *call_flags = flags;
                    }
                }

                #fn_instance
            }

            impl #impl_type {
                #(pub #method_sigs {
                        match self {
                            #impl_type::Instance { address, call_flags } => {
                                use #crate_ink_lang::codegen::TraitCallBuilder;
                                let mut forwarder = Forwarder::from_account_id(*address);
                                forwarder
                                    .#call_fns()
                                    .#method_forward_calls
                                    .call_flags(call_flags.clone())
                                    .invoke()
                            }
                            #[cfg(feature = "std")]
                            #impl_type::Mock => {
                                MOCK.with(move |x| {
                                    let mut borrow = x.borrow_mut();
                                    let (callee, forwarder) = borrow.as_mut().unwrap();
                                    let prev_callee = #crate_ink_env::test::callee::<PinkEnvironment>();
                                    let prev_caller = #crate_ink_env::caller::<PinkEnvironment>();
                                    #crate_ink_env::test::set_caller::<PinkEnvironment>(prev_callee.clone());
                                    #crate_ink_env::test::set_callee::<PinkEnvironment>(callee.clone());
                                    let ret = forwarder.#method_forward_calls;
                                    #crate_ink_env::test::set_callee::<PinkEnvironment>(prev_callee);
                                    #crate_ink_env::test::set_caller::<PinkEnvironment>(prev_caller);
                                    ret
                                })
                            }
                        }
                    }
                )*
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn show_patch_result() {
        let stream = patch(
            syn::parse_quote! {
                #[ink::trait_definition(namespace = "pink_system")]
                pub trait System {
                    #[ink(message)]
                    fn get_driver(&self, name: String) -> Option<AccountId>;

                    #[ink(message)]
                    fn set_driver(&self, name: String, driver: AccountId);

                    #[ink(message)]
                    fn deploy_sidevm_to(&self, code_hash: Hash, contract_id: AccountId) -> Result<()>;
                }
            },
            syn::parse_quote!(),
            InterfaceType::System,
        );
        insta::assert_display_snapshot!(rustfmt_snippet::rustfmt_token_stream(&stream).unwrap());
    }
}
