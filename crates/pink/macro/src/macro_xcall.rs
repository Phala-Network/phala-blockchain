use proc_macro2::{Ident, Literal, Span, TokenStream};
use syn::{parse_quote, Result};

struct Method {
    id: u32,
    args: Vec<TokenStream>,
    method: syn::TraitItemMethod,
}

impl Method {
    fn parse(method: &syn::TraitItemMethod) -> Result<Self> {
        let mut id = None;

        for attr in method.attrs.iter() {
            if !attr.is_xcall() {
                continue;
            }
            match attr.parse_meta()? {
                syn::Meta::List(list) => {
                    for nested in list.nested.iter() {
                        match nested {
                            syn::NestedMeta::Meta(syn::Meta::NameValue(name_value)) => {
                                match name_value
                                    .path
                                    .get_ident()
                                    .ok_or_else(|| {
                                        syn::Error::new_spanned(
                                            &name_value.path,
                                            "Expected an identifier",
                                        )
                                    })?
                                    .to_string()
                                    .as_str()
                                {
                                    "id" => match &name_value.lit {
                                        syn::Lit::Int(value) => {
                                            let parsed_id = value.base10_parse::<u32>()?;
                                            if parsed_id == 0 {
                                                return Err(syn::Error::new_spanned(
                                                    &name_value.lit,
                                                    "Id must be non-zero",
                                                ));
                                            }
                                            id = Some(parsed_id);
                                        }
                                        _ => {
                                            return Err(syn::Error::new_spanned(
                                                &name_value.lit,
                                                "Expected an integer",
                                            ));
                                        }
                                    },
                                    attr => {
                                        return Err(syn::Error::new_spanned(
                                            name_value,
                                            format!("Unknown attribute: {attr}"),
                                        ));
                                    }
                                }
                            }
                            _ => {
                                return Err(syn::Error::new_spanned(nested, "Invalid attribute"));
                            }
                        }
                    }
                }
                _ => {
                    let err = syn::Error::new_spanned(attr, "call attribute must be a list");
                    return Err(err);
                }
            }
        }

        match id {
            None => Err(syn::Error::new_spanned(
                &method.sig,
                "Missing call id attribute",
            )),
            Some(id) => {
                let mut method = method.clone();
                let args = parse_args(&method)?;
                method.attrs.retain(|attr| !attr.is_xcall());
                Ok(Method { id, args, method })
            }
        }
    }
}

pub(crate) fn patch(config: TokenStream, input: TokenStream) -> TokenStream {
    match patch_or_err(config, input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
}

fn patch_or_err(config: TokenStream, input: TokenStream) -> Result<TokenStream> {
    let impl_for: syn::Ident = syn::parse2(config)?;
    let trait_item: syn::ItemTrait = syn::parse2(input)?;
    let call_methods: Result<Vec<Method>> = trait_item
        .items
        .iter()
        .filter_map(|item| {
            if let syn::TraitItem::Method(method) = item {
                Some(Method::parse(method))
            } else {
                None
            }
        })
        .collect();
    let call_methods = call_methods?;
    check_redundant_call_id(&call_methods)?;
    check_args_multi_ref(&call_methods)?;

    let trait_item = patch_call_trait(trait_item);

    let call_impl = gen_call_impl(&call_methods, &trait_item.ident, &impl_for)?;
    let trait_ro = gen_call_impl_ro(&call_methods, &trait_item.ident, &impl_for)?;

    let exec_dispatcher = gen_exec_dispatcher(&call_methods, &trait_item.ident)?;

    let id2name = gen_id2name(&call_methods)?;

    Ok(parse_quote! {
        #trait_item

        #call_impl

        #trait_ro

        #exec_dispatcher

        #id2name
    })
}

fn gen_exec_dispatcher(call_methods: &[Method], trait_name: &Ident) -> Result<TokenStream> {
    let mut calls: Vec<TokenStream> = Vec::new();

    for method in call_methods {
        let id = Literal::u32_unsuffixed(method.id);
        let sig = &method.method.sig;
        let name = &sig.ident;
        let args = &method.args;
        let exec_fn = if is_mut_self(method) {
            Ident::new("execute_mut", Span::call_site())
        } else {
            Ident::new("execute", Span::call_site())
        };

        calls.push(parse_quote! {
            #id => {
                let (#(#args),*) = Decode::decode(&mut &input[..]).expect("Failed to decode args");
                executor.#exec_fn(move || srv.#name(#(#args),*)).encode()
            }
        });
    }

    Ok(parse_quote! {
        pub fn executing_dispatch(
            executor: &mut (impl Executing + ?Sized),
            srv: &mut (impl #trait_name + ?Sized),
            id: u32,
            input: &[u8],
        ) -> Vec<u8> {
            match id {
                #(#calls)*
                _ => panic!("Unknown call id {}", id),
            }
        }
    })
}

fn gen_id2name(methods: &[Method]) -> Result<TokenStream> {
    let (ids, names): (Vec<_>, Vec<_>) = methods
        .iter()
        .map(|m| (m.id, Literal::string(&m.method.sig.ident.to_string())))
        .unzip();
    Ok(parse_quote! {
        pub fn id2name(id: u32) -> &'static str {
            match id {
                #(#ids => #names,)*
                _ => "unknown",
            }
        }
    })
}

fn parse_args(method: &syn::TraitItemMethod) -> Result<Vec<TokenStream>> {
    method
        .sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(arg) = arg {
                if let syn::Pat::Ident(ident) = &*arg.pat {
                    Some(Ok(parse_quote!(#ident)))
                } else {
                    Some(Err(syn::Error::new_spanned(
                        &arg.pat,
                        "Expected an identifier",
                    )))
                }
            } else {
                None
            }
        })
        .collect()
}

fn gen_call_impl(
    call_methods: &[Method],
    trait_name: &Ident,
    impl_for: &Ident,
) -> Result<TokenStream> {
    let impl_methods: Result<Vec<TokenStream>> =
        call_methods.iter().map(gen_call_impl_method).collect();
    let impl_methods = impl_methods?;
    Ok(parse_quote! {
        impl<T: #impl_for + CrossCallMut> #trait_name for T {
            #(#impl_methods)*
        }
    })
}

fn gen_call_impl_ro(
    call_methods: &[Method],
    trait_name: &Ident,
    impl_for: &Ident,
) -> Result<TokenStream> {
    let methods: Vec<_> = call_methods.iter().filter(|m| !is_mut_self(m)).collect();
    let trait_methods: Vec<_> = methods.iter().map(|m| m.method.clone()).collect();
    let impl_methods: Result<Vec<TokenStream>> =
        methods.into_iter().map(gen_call_impl_method).collect();
    let impl_methods = impl_methods?;
    let trait_ro = Ident::new(&format!("{}Ro", trait_name), Span::call_site());
    Ok(parse_quote! {
        pub trait #trait_ro {
            #(#trait_methods)*
        }
        impl<T: #impl_for> #trait_ro for T {
            #(#impl_methods)*
        }
    })
}

fn is_mut_self(method: &Method) -> bool {
    match method.method.sig.inputs.first() {
        Some(syn::FnArg::Receiver(recv)) => recv.mutability.is_some(),
        _ => false,
    }
}

fn gen_call_impl_method(method: &Method) -> Result<TokenStream> {
    let sig = method.method.sig.clone();
    let call_id = Literal::u32_unsuffixed(method.id);
    let args = &method.args;
    let cross_fn = if is_mut_self(method) {
        Ident::new("cross_call_mut", Span::call_site())
    } else {
        Ident::new("cross_call", Span::call_site())
    };
    Ok(parse_quote! {
        #sig {
            let inputs = (#(#args),*);
            let ret = self.#cross_fn(#call_id, &inputs.encode());
            Decode::decode(&mut &ret[..]).expect("Decode failed")
        }
    })
}

fn check_redundant_call_id(methods: &[Method]) -> Result<()> {
    let mut ids = Vec::new();
    for method in methods {
        if ids.contains(&method.id) {
            return Err(syn::Error::new_spanned(
                &method.method.sig,
                format!("Duplicate call id: {}", method.id),
            ));
        }
        ids.push(method.id);
    }
    Ok(())
}

fn check_args_multi_ref(methods: &[Method]) -> Result<()> {
    for method in methods {
        let mut n_ref = 0;
        let mut has_mut_ref = false;
        for arg in method.method.sig.inputs.iter() {
            let syn::FnArg::Typed(arg) = arg else {
                continue;
            };
            let syn::Type::Reference(ty) = &*arg.ty else {
                continue;
            };
            n_ref += 1;
            if ty.mutability.is_some() {
                has_mut_ref = true;
            }
        }
        if has_mut_ref && n_ref > 1 {
            return Err(syn::Error::new_spanned(
                &method.method.sig,
                "Only one &mut ref argument is allowed",
            ));
        }
    }
    Ok(())
}

fn patch_call_trait(mut input: syn::ItemTrait) -> syn::ItemTrait {
    for item in input.items.iter_mut() {
        if let syn::TraitItem::Method(method) = item {
            // Remove the call attribute
            method.attrs.retain(|attr| !attr.is_xcall());
            // Add &mut self as receiver
            if !matches!(&method.sig.inputs.first(), Some(syn::FnArg::Receiver(_))) {
                method.sig.inputs.insert(
                    0,
                    parse_quote! {
                        &mut self
                    },
                );
            }
        }
    }
    input
}

trait AttributeExt {
    fn is_xcall(&self) -> bool;
}

impl AttributeExt for syn::Attribute {
    fn is_xcall(&self) -> bool {
        self.path.is_ident("xcall")
    }
}
