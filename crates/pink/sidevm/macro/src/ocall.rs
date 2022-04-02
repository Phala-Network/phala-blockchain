use proc_macro2::{Ident, Literal, Span, TokenStream};
use syn::{parse_quote, Result};

struct OcallMethod {
    id: i32,
    fast_return: bool,
    fast_input: bool,
    args: Vec<TokenStream>,
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
                args: parse_args(method)?,
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

    let trait_item = remove_ocall_attributes(trait_item);

    let ocall_impl = gen_ocall_impl(&ocall_methods, &trait_item.ident)?;

    let dispatcher = gen_dispatcher(&ocall_methods, &trait_item.ident)?;

    let id2name = gen_id2name(&ocall_methods)?;

    Ok(parse_quote! {
        #trait_item

        #ocall_impl

        #dispatcher

        #id2name
    })
}

fn args_r(n: usize) -> Vec<Ident> {
    (0..n)
        .map(|i| Ident::new(&format!("p{}", i), Span::call_site()))
        .collect()
}

fn gen_dispatcher(methods: &[OcallMethod], trait_name: &Ident) -> Result<TokenStream> {
    let mut fast_calls: Vec<TokenStream> = Vec::new();
    let mut slow_calls: Vec<TokenStream> = Vec::new();

    for method in methods {
        let id = Literal::i32_unsuffixed(method.id);
        let name = &method.method.sig.ident;
        let args = &method.args;
        let parse_inputs: TokenStream = if method.fast_input {
            let args_r = args_r(args.len());
            parse_quote! {
                let (#(#args),*) = (#(#args_r as _),*);
            }
        } else {
            parse_quote! {
                let (#(#args),*) = {
                    env.with_slice_from_vm(p0, p1, |mut buf| {
                        Decode::decode(&mut buf)
                    })?
                    .or(Err(OcallError::InvalidParameter))?
                };
            }
        };
        let calling: TokenStream = parse_quote! {
            env.#name(#(#args),*)
        };

        if method.fast_return {
            fast_calls.push(parse_quote! {
                #id => {
                    #parse_inputs
                    #calling as _
                }
            });
        };

        slow_calls.push(parse_quote! {
            #id => {
                #parse_inputs
                let ret = #calling;
                env.put_return(ret.encode()) as _
            }
        });
    }

    let call_get_return: TokenStream = parse_quote! {
        {
            let buffer = env.take_return().ok_or(OcallError::NoReturnValue)?;
            let len = p1 as usize;
            if buffer.len() != len {
                return Err(OcallError::InvalidParameter);
            }
            env.copy_to_vm(&buffer, p0)?;
            len as IntPtr
        }
    };

    Ok(parse_quote! {
        pub fn dispatch_call_fast_return<Env: #trait_name + OcallEnv>(
            env: &Env,
            id: i32,
            p0: IntPtr,
            p1: IntPtr,
            p2: IntPtr,
            p3: IntPtr
        ) -> Result<IntPtr> {
            Ok(match id {
                0 => #call_get_return,
                #(#fast_calls)*
                _ => return Err(OcallError::UnknownCallNumber),
            })
        }

        pub fn dispatch_call<Env: #trait_name + OcallEnv>(
            env: &Env,
            id: i32,
            p0: IntPtr,
            p1: IntPtr,
            p2: IntPtr,
            p3: IntPtr
        ) -> Result<IntPtr> {
            Ok(match id {
                0 => {
                    let ret: IntPtr = #call_get_return;
                    env.put_return(ret.encode()) as _
                }
                #(#slow_calls)*
                _ => return Err(OcallError::UnknownCallNumber),
            })
        }

        pub trait OcallEnv {
            fn put_return(&self, rv: Vec<u8>) -> usize;
            fn take_return(&self) -> Option<Vec<u8>>;
            fn copy_to_vm(&self, data: &[u8], ptr: IntPtr) -> Result<()>;
            fn with_slice_from_vm<T>(
                &self,
                ptr: IntPtr,
                len: IntPtr,
                f: impl FnOnce(&[u8]) -> T
            ) -> Result<T>;
        }
    })
}

fn gen_id2name(methods: &[OcallMethod]) -> Result<TokenStream> {
    let (ids, names): (Vec<_>, Vec<_>) = methods
        .iter()
        .map(|m| (m.id, Literal::string(&m.method.sig.ident.to_string())))
        .unzip();
    Ok(parse_quote! {
        pub fn ocall_id2name(id: i32) -> &'static str {
            match id {
                0 => "get_return",
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

fn pad_args(args: &[TokenStream]) -> Result<Vec<TokenStream>> {
    let ocall_nargs = 4;
    if args.len() > ocall_nargs {
        return Err(syn::Error::new_spanned(&args[0], "Too many arguments"));
    }
    let mut args: Vec<_> = args.iter().cloned().collect();
    for arg in args.iter_mut() {
        *arg = parse_quote!( #arg as _ );
    }
    if args.len() < ocall_nargs {
        for _ in args.len()..ocall_nargs {
            args.push(parse_quote! { 0 });
        }
    }
    Ok(args)
}

fn gen_ocall_impl(ocall_methods: &[OcallMethod], trait_name: &Ident) -> Result<TokenStream> {
    let impl_methods: Result<Vec<TokenStream>> = ocall_methods
        .iter()
        .map(|method| gen_ocall_impl_method(method))
        .collect();

    let impl_itent = Ident::new(&format!("{}Implement", trait_name), Span::call_site());
    let impl_methods = impl_methods?;
    Ok(parse_quote! {
        pub struct #impl_itent;
        impl #impl_itent {
            #(#impl_methods)*
        }
    })
}

fn gen_ocall_impl_method(method: &OcallMethod) -> Result<TokenStream> {
    let sig = &method.method.sig;

    let call_id = Literal::i32_unsuffixed(method.id);

    let args = if method.fast_input {
        pad_args(&method.args)?
    } else {
        method.args.clone()
    };

    let ocall_fn = if method.fast_return {
        "sidevm_ocall_fast_return"
    } else {
        "sidevm_ocall"
    };
    let ocall_fn = Ident::new(ocall_fn, Span::call_site());

    let body_top: TokenStream = if method.fast_input {
        parse_quote! {
            let ret = #ocall_fn(current_task(), #call_id, #(#args),*);
        }
    } else {
        parse_quote! {
            let inputs = (#(#args),*);
            let mut input_buf = empty_buffer();
            Encode::encode_to(&inputs, &mut input_buf);
            let len = input_buf.len() as IntPtr;
            let ret = #ocall_fn(
                current_task(),
                #call_id,
                input_buf.as_ptr() as IntPtr,
                len,
                0,
                0
            );
        }
    };

    let body_bottom: TokenStream = if method.fast_return {
        parse_quote!(ret as _)
    } else {
        parse_quote! {
            let len = ret;
            if len < 0 {
                panic!("ocall returned an error");
            }
            let mut buf = alloc_buffer(len as _);
            let ret = sidevm_ocall_fast_return(
                current_task(),
                0, // Get previous ocall's output
                buf.as_mut_ptr() as IntPtr,
                len,
                0,
                0
            );
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
            fn call_slow(&self, a: i32, b: i32) -> i32;

            #[ocall(id = 103, fast_input)]
            fn call_fi(&self, a: i32, b: i32) -> i32;

            #[ocall(id = 104, fast_return)]
            fn call_fo(&self, a: i32, b: i32) -> i32;

            #[ocall(id = 102, fast_input, fast_return)]
            fn poll_fi_fo(&self, a: i32, b: i32) -> i32;
        }
    });
    insta::assert_display_snapshot!(rustfmt_snippet::rustfmt_token_stream(&stream).unwrap())
}
