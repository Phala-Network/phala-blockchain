use proc_macro2::{Ident, Literal, Span, TokenStream};
use syn::{parse_quote, Result};

struct OcallMethod {
    id: i32,
    fast_return: bool,
    fast_input: bool,
    method: syn::TraitItemMethod,
}

impl OcallMethod {
    fn parse(method: &syn::TraitItemMethod) -> Result<Self> {
        let mut id = None;
        let mut fast_return = false;
        let mut fast_input = false;

        for attr in method.attrs.iter() {
            if attr.is_ocall() {
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
                                                let parsed_id = value.base10_parse::<i32>()?;
                                                if parsed_id < 100 {
                                                    return Err(syn::Error::new_spanned(
                                                        &name_value.lit,
                                                        "Id must greater than 100",
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
                                                format!("Unknown attribute: {}", attr),
                                            ));
                                        }
                                    }
                                }
                                syn::NestedMeta::Meta(syn::Meta::Path(path)) => {
                                    match path
                                        .get_ident()
                                        .ok_or_else(|| {
                                            syn::Error::new_spanned(path, "Expected an identifier")
                                        })?
                                        .to_string()
                                        .as_str()
                                    {
                                        "fast_return" => fast_return = true,
                                        "fast_input" => fast_input = true,
                                        attr => {
                                            return Err(syn::Error::new_spanned(
                                                path,
                                                format!("Unknown attribute: {}", attr),
                                            ));
                                        }
                                    }
                                }
                                _ => {
                                    return Err(syn::Error::new_spanned(
                                        nested,
                                        "Invalid attribute",
                                    ));
                                }
                            }
                        }
                    }
                    _ => {
                        let err = syn::Error::new_spanned(attr, "ocall attribute must be a list");
                        return Err(err);
                    }
                }
            }
        }

        match id {
            None => {
                return Err(syn::Error::new_spanned(
                    &method.sig,
                    "Missing ocall id attribute",
                ))
            }
            Some(id) => Ok(OcallMethod {
                id,
                fast_return,
                fast_input,
                method: method.clone(),
            }),
        }
    }
}

pub(crate) fn patch(input: TokenStream) -> TokenStream {
    match patch_or_err(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
}

fn patch_or_err(input: TokenStream) -> Result<TokenStream> {
    let trait_item: syn::ItemTrait = syn::parse2(input)?;
    let impl_itent = Ident::new(&format!("{}Implement", trait_item.ident), Span::call_site());

    let ocall_methods: Result<Vec<OcallMethod>> = trait_item
        .items
        .iter()
        .filter_map(|item| {
            if let syn::TraitItem::Method(method) = item {
                Some(OcallMethod::parse(method))
            } else {
                None
            }
        })
        .collect();
    let ocall_methods = ocall_methods?;
    check_redundant_ocall_id(&ocall_methods)?;

    let impl_methods: Result<Vec<TokenStream>> = ocall_methods
        .iter()
        .map(|method| gen_ocall_impl(method))
        .collect();

    let impl_methods = impl_methods?;

    let trait_item = remove_ocall_attributes(trait_item);

    Ok(parse_quote! {
        #trait_item

        pub struct #impl_itent;
        impl #impl_itent {
            #(#impl_methods)*
        }
    })
}

fn gen_ocall_impl(method: &OcallMethod) -> Result<TokenStream> {
    let sig = &method.method.sig;

    fn pad_args(mut args: Vec<TokenStream>) -> Result<Vec<TokenStream>> {
        let ocall_nargs = 4;
        if args.len() > ocall_nargs {
            return Err(syn::Error::new_spanned(&args[0], "Too many arguments"));
        }
        if args.len() < ocall_nargs {
            for _ in args.len()..ocall_nargs {
                args.push(parse_quote! { 0 });
            }
        }
        Ok(args)
    }

    let args: Result<Vec<TokenStream>> = sig
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
        .collect();

    let args = args?;

    let call_id = Literal::i32_unsuffixed(method.id);

    let args = if method.fast_input {
        pad_args(args)?
    } else {
        args
    };

    let ocall_fn = if method.fast_return {
        "sidevm_ocall_fast"
    } else {
        "sidevm_ocall"
    };
    let ocall_fn = Ident::new(ocall_fn, Span::call_site());

    let body_top: TokenStream = if method.fast_input {
        parse_quote! {
            let ret = #ocall_fn(#call_id, #(#args),*);
        }
    } else {
        parse_quote! {
            let inputs = (#(#args),*);
            let mut input_buf = empty_buffer();
            Encode::encode_to(&inputs, &mut input_buf);
            let len = input_buf.len() as IntPtr;
            let ret = sidevm_ocall(#call_id, input_buf.as_ptr() as IntPtr, len, 0, 0);
        }
    };

    let body_bottom: TokenStream = if method.fast_return {
        parse_quote!(ret)
    } else {
        parse_quote! {
            let len = ret;
            if len < 0 {
                panic!("ocall returned an error");
            }
            let mut buf = alloc_buffer(len as _);
            let ret = sidevm_ocall_fast(0, buf.as_mut_ptr() as IntPtr, len, 0, 0);
            if ret != len {
                panic!("ocall get return length mismatch");
            }
            Decode::decode(&mut buf.as_ref()).expect("Failed to decode ocall return value")
        }
    };

    Ok(parse_quote! {
        pub #sig {
            unsafe {
                #body_top
                #body_bottom
            }
        }
    })
}

fn check_redundant_ocall_id(methods: &[OcallMethod]) -> Result<()> {
    let mut ids = Vec::new();
    for method in methods {
        if ids.contains(&method.id) {
            return Err(syn::Error::new_spanned(
                &method.method.sig,
                format!("Duplicate ocall id: {}", method.id),
            ));
        }
        ids.push(method.id);
    }
    Ok(())
}

fn remove_ocall_attributes(mut input: syn::ItemTrait) -> syn::ItemTrait {
    for item in input.items.iter_mut() {
        if let syn::TraitItem::Method(method) = item {
            method.attrs.retain(|attr| !attr.is_ocall());
        }
    }
    input
}

trait AttributeExt {
    fn is_ocall(&self) -> bool;
}

impl AttributeExt for syn::Attribute {
    fn is_ocall(&self) -> bool {
        self.path.is_ident("ocall")
    }
}

#[test]
fn test() {
    let stream = patch(parse_quote! {
        pub trait Ocall {
            #[ocall(id = 101)]
            fn call_slow(&self, p0: i32, p1: i32) -> i32;

            #[ocall(id = 103, fast_input)]
            fn call_fi(&self, p0: i32, p1: i32) -> i32;

            #[ocall(id = 104, fast_return)]
            fn call_fo(&self, p0: i32, p1: i32) -> i32;

            #[ocall(id = 102, fast_input, fast_return)]
            fn poll_fi_fo(&self, p0: i32, p1: i32) -> i32;

        }
    });
    println!(
        "{}",
        rustfmt_snippet::rustfmt_token_stream(&stream).unwrap()
    );
}
