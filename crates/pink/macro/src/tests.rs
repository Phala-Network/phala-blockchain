#[cfg(all(target_arch="x86_64", target_os = "linux"))]
#[test]
fn test_xcall() {
    let stream = crate::macro_xcall::patch(syn::parse_quote!(Impl), syn::parse_quote! {
        pub trait ECalls {
            #[xcall(id = 1)]
            fn instantiate(
                &self,
                code_hash: Hash,
                endowment: Balance,
                gas_limit: u64,
                input_data: Vec<u8>,
            ) -> Result<Hash, u32>;
            #[xcall(id = 2)]
            fn set_id(
                &mut self,
                id: Hash,
            ) -> Result<Hash, u32>;
        }
    });
    insta::assert_display_snapshot!(rustfmt_snippet::rustfmt_token_stream(&stream).unwrap())
}
