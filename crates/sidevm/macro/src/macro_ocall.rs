use heck::ToSnakeCase;
use proc_macro2::{Ident, Literal, Span, TokenStream};
use syn::{parse_quote, Result};

struct OcallMethod {
    id: i32,
    encode_output: bool,
    encode_input: bool,
    args: Vec<TokenStream>,
    method: syn::TraitItemMethod,
}

impl OcallMethod {
    fn parse(method: &syn::TraitItemMethod) -> Result<Self> {
        let mut id = None;
        let mut encode_output = false;
        let mut encode_input = false;

        for attr in method.attrs.iter() {
            if !attr.is_ocall() {
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
                                    "encode_input" => encode_input = true,
                                    "encode_output" => encode_output = true,
                                    attr => {
                                        return Err(syn::Error::new_spanned(
                                            path,
                                            format!("Unknown attribute: {}", attr),
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
                    let err = syn::Error::new_spanned(attr, "ocall attribute must be a list");
                    return Err(err);
                }
            }
        }

        match id {
            None => Err(syn::Error::new_spanned(
                &method.sig,
                "Missing ocall id attribute",
            )),
            Some(id) => Ok(OcallMethod {
                id,
                encode_input,
                encode_output,
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
    check_args_multi_ref(&ocall_methods)?;

    let trait_item = patch_ocall_trait(trait_item);

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

fn gen_dispatcher(methods: &[OcallMethod], trait_name: &Ident) -> Result<TokenStream> {
    let mut fast_calls: Vec<TokenStream> = Vec::new();
    let mut slow_calls: Vec<TokenStream> = Vec::new();

    for method in methods {
        let id = Literal::i32_unsuffixed(method.id);
        let name = &method.method.sig.ident;
        let args = &method.args;
        let args_reversed = args.iter().rev();
        let parse_inputs: TokenStream = if !method.encode_input {
            parse_quote! {
                let stack = StackedArgs::load(&[p0, p1, p2, p3]).ok_or(OcallError::InvalidParameter)?;
                #(let (#args_reversed, stack) = stack.pop_arg(vm)?;)*
                let _: StackedArgs<()> = stack;
            }
        } else {
            parse_quote! {
                let (#(#args),*) = {
                    let mut buf = vm.slice_from_vm(p0, p1)?;
                    Decode::decode(&mut buf).or(Err(OcallError::InvalidParameter))?
                };
            }
        };
        let calling: TokenStream = parse_quote! {
            env.#name(#(#args),*)
        };

        if !method.encode_output {
            fast_calls.push(parse_quote! {
                #id => {
                    #parse_inputs
                    #calling.map(|x| x.to_i32())
                }
            });
        } else {
            slow_calls.push(parse_quote! {
                #id => {
                    #parse_inputs
                    let ret = #calling;
                    env.put_return(ret?.encode()) as _
                }
            });
        }
    }

    let call_get_return: TokenStream = parse_quote! {
        {
            let buffer = env.take_return().ok_or(OcallError::NotFound)?;
            let len = p1 as usize;
            if buffer.len() != len {
                return Err(OcallError::InvalidParameter);
            }
            vm.copy_to_vm(&buffer, p0)?;
            Ok(len as i32)
        }
    };

    Ok(parse_quote! {
        pub fn dispatch_call_fast_return<Env: #trait_name + OcallEnv, Vm: VmMemory>(
            env: &mut Env,
            vm: &Vm,
            id: i32,
            p0: IntPtr,
            p1: IntPtr,
            p2: IntPtr,
            p3: IntPtr
        ) -> Result<i32> {
            match id {
                0 => #call_get_return,
                #(#fast_calls)*
                _ => Err(OcallError::UnknownCallNumber),
            }
        }

        pub fn dispatch_call<Env: #trait_name + OcallEnv, Vm: VmMemory>(
            env: &mut Env,
            vm: &Vm,
            id: i32,
            p0: IntPtr,
            p1: IntPtr,
            p2: IntPtr,
            p3: IntPtr
        ) -> Result<i32> {
            Ok(match id {
                #(#slow_calls)*
                _ => return Err(OcallError::UnknownCallNumber),
            })
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

fn gen_ocall_impl(ocall_methods: &[OcallMethod], trait_name: &Ident) -> Result<TokenStream> {
    let impl_methods: Result<Vec<TokenStream>> =
        ocall_methods.iter().map(gen_ocall_impl_method).collect();

    let name = format!("{}_guest", trait_name.to_string().to_snake_case());
    let impl_ident = Ident::new(&name, Span::call_site());
    let impl_methods = impl_methods?;

    Ok(parse_quote! {
        pub mod #impl_ident {
            use super::*;

            #(#impl_methods)*
        }
    })
}

fn gen_ocall_impl_method(method: &OcallMethod) -> Result<TokenStream> {
    let mut sig = method.method.sig.clone();

    if matches!(
        method.method.sig.inputs.first(),
        Some(syn::FnArg::Receiver(_))
    ) {
        let mut inputs = sig.inputs.clone();
        sig.inputs.clear();
        while let Some(arg) = inputs.pop() {
            let arg = arg.into_value();
            if let syn::FnArg::Receiver(_) = arg {
                break;
            }
            sig.inputs.insert(0, arg)
        }
    };

    let call_id = Literal::i32_unsuffixed(method.id);

    let args = &method.args;

    let ocall_fn = if method.encode_output {
        "do_ocall"
    } else {
        "do_ocall_fast_return"
    };
    let ocall_fn = Ident::new(ocall_fn, Span::call_site());

    let body_top: TokenStream = if !method.encode_input {
        parse_quote! {
            let stack = StackedArgs::empty();
            #(let stack = stack.push_arg(#args);)*
            let args = stack.dump();
            let ret = #ocall_fn(#call_id, args[0], args[1], args[2], args[3]);
        }
    } else {
        parse_quote! {
            let inputs = (#(#args),*);
            let mut input_buf = Buffer::default();
            Encode::encode_to(&inputs, &mut input_buf);
            let len = input_buf.len() as IntPtr;
            let ret = #ocall_fn(
                #call_id,
                input_buf.as_ptr() as IntPtr,
                len,
                0,
                0
            );
        }
    };

    let body_bottom: TokenStream = if !method.encode_output {
        parse_quote!(<Result<i32> as RetDecode>::decode_ret(ret).and_then(I32Convertible::from_i32))
    } else {
        parse_quote! {
            let len = <Result<i32> as RetDecode>::decode_ret(ret)?;
            if len < 0 {
                panic!("ocall returned an error");
            }
            let mut buf = alloc_buffer(len as _);
            let ret = do_ocall_fast_return(
                0, // Get previous ocall's output
                buf.as_mut_ptr() as IntPtr,
                len as IntPtr,
                0,
                0
            );
            let ret = <Result<i32> as RetDecode>::decode_ret(ret)?;
            if ret != len {
                panic!("ocall get return length mismatch");
            }
            Ok(Decode::decode(&mut buf.as_ref()).expect("Failed to decode ocall return value"))
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

fn check_args_multi_ref(methods: &[OcallMethod]) -> Result<()> {
    for method in methods {
        let mut n_ref = 0;
        let mut has_mut_ref = false;
        for arg in method.method.sig.inputs.iter() {
            // TODO: use `let else`
            if let syn::FnArg::Typed(arg) = arg {
                if let syn::Type::Reference(ty) = &*arg.ty {
                    n_ref += 1;
                    if ty.mutability.is_some() {
                        has_mut_ref = true;
                    }
                }
            }
        }
        if has_mut_ref && n_ref > 1 {
            return Err(syn::Error::new_spanned(
                &method.method.sig,
                format!("Only one &mut ref argument is allowed"),
            ));
        }
    }
    Ok(())
}

fn patch_ocall_trait(mut input: syn::ItemTrait) -> syn::ItemTrait {
    for item in input.items.iter_mut() {
        if let syn::TraitItem::Method(method) = item {
            // Remove the ocall attribute
            method.attrs.retain(|attr| !attr.is_ocall());
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
    fn is_ocall(&self) -> bool;
}

impl AttributeExt for syn::Attribute {
    fn is_ocall(&self) -> bool {
        self.path.is_ident("ocall")
    }
}
