#[test]
fn test_ocall() {
    let stream = crate::macro_ocall::patch(syn::parse_quote! {
        pub trait Ocall {
            #[ocall(id = 101)]
            fn call_slow(a: i32, b: i32) -> i32;

            #[ocall(id = 103, fast_input)]
            fn call_fi(a: i32, b: i32) -> i32;

            #[ocall(id = 104, fast_return)]
            fn call_fo(a: i32, b: i32) -> i32;

            #[ocall(id = 102, fast_input, fast_return)]
            fn poll_fi_fo(a: i32, b: i32) -> i32;
        }
    });
    insta::assert_display_snapshot!(rustfmt_snippet::rustfmt_token_stream(&stream).unwrap())
}

#[test]
fn test_main() {
    let stream = crate::macro_main::patch(syn::parse_quote! {
        async fn the_main() {
            sleep(1).await
        }
    });
    insta::assert_display_snapshot!(rustfmt_snippet::rustfmt_token_stream(&stream).unwrap())
}
