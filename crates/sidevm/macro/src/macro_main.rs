use proc_macro2::TokenStream;

pub(crate) fn patch(input: TokenStream) -> TokenStream {
    match patch_or_err(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
}

fn patch_or_err(input: TokenStream) -> syn::Result<TokenStream> {
    let main_fn: syn::ItemFn = syn::parse2(input.clone())?;
    let main_ident = &main_fn.sig.ident;
    let crate_sidevm = crate::find_crate_name("sidevm")?;
    Ok(syn::parse_quote! {
        #[no_mangle]
        extern "C" fn sidevm_poll() -> i32 {
            #crate_sidevm::env::tasks::sidevm_poll()
        }
        #[no_mangle]
        fn sidevm_main_future() -> std::pin::Pin<std::boxed::Box<dyn std::future::Future<Output = ()>>> {

            #input

            Box::pin(#main_ident())
        }
    })
}
