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
    let sidevm_crate = crate::find_crate_name("pink-sidevm")?;
    Ok(syn::parse_quote! {
        #[no_mangle]
        extern "C" fn sidevm_poll() -> i32 {
            use #sidevm_crate::env::{poll_with_dummy_context, reexports::once_cell::sync::Lazy};
            use std::{future::Future, pin::Pin, sync::Mutex, task::Poll, cell::RefCell};

            #input

            thread_local! {
                static MAIN_FUTURE: Lazy<RefCell<Pin<Box<dyn Future<Output = ()>>>>> =
                    Lazy::new(|| RefCell::new(Box::pin(#main_ident())));
            }

            MAIN_FUTURE.with(|cell| {
                match poll_with_dummy_context(cell.borrow_mut().as_mut()) {
                    Poll::Ready(()) => 1,
                    Poll::Pending => 0,
                }
            })
        }
    })
}
