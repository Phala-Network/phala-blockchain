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
    let pink_env_crate = crate::find_crate_name("pink-sidevm-env")?;
    Ok(syn::parse_quote! {
        #[no_mangle]
        extern "C" fn sidevm_poll() -> i32 {
            use #pink_env_crate::{poll_with_dummy_context, reexports::once_cell::sync::Lazy};
            use std::{future::Future, pin::Pin, sync::Mutex, task::Poll};

            #input

            static MAIN_FUTURE: Lazy<Mutex<Pin<Box<dyn Future<Output = ()> + Sync + Send>>>> =
                Lazy::new(|| Mutex::new(Box::pin(#main_ident())));

            match poll_with_dummy_context(MAIN_FUTURE.lock().unwrap().as_mut()) {
                Poll::Ready(()) => 1,
                Poll::Pending => 0,
            }
        }
    })
}
