macro_rules! snapshot_with_tokens {
        ($($tokens:tt)*) => {
            let stream = crate::macro_xcall::patch(
                syn::parse_quote!(Impl),
                syn::parse_quote! {
                    $($tokens)*
                },
            );
            insta::assert_display_snapshot!(rustfmt_snippet::rustfmt_token_stream(&stream).unwrap())
        };
    }
#[test]
fn test_xcall() {
    snapshot_with_tokens! {
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
    }
}

#[test]
fn test_attr_not_a_list() {
    snapshot_with_tokens! {
        pub trait ECalls {
            #[xcall]
            fn foo(&self);
        }
    }
}

#[test]
fn test_missing_call_id() {
    snapshot_with_tokens! {
        pub trait ECalls {
            #[xcall()]
            fn foo(&self);
        }
    }
}

#[test]
fn test_non_method_item() {
    snapshot_with_tokens! {
        pub trait ECalls {
            type Foo;
            #[xcall(id=1)]
            fn foo(&self);
        }
    }
}

#[test]
fn test_invalid_attr_tokens() {
    snapshot_with_tokens! {
        pub trait ECalls {
            #[xcall(s.b%)]
            fn foo();
        }
    }
}

#[test]
fn test_invalid_attr() {
    snapshot_with_tokens! {
        pub trait ECalls {
            #[xcall(123)]
            fn foo(&self);
        }
    }
}

#[test]
fn test_attr_not_ident() {
    snapshot_with_tokens! {
        pub trait ECalls {
            #[xcall(foo::bar=42)]
            fn foo(&self);
        }
    }
}

#[test]
fn test_invalid_id_value() {
    snapshot_with_tokens! {
        pub trait ECalls {
            #[xcall(id="foo")]
            fn foo(&self);
        }
    }
}

#[test]
fn test_zero_id() {
    snapshot_with_tokens! {
        pub trait ECalls {
            #[xcall(id=0)]
            fn foo(&self);
        }
    }
}

#[test]
fn test_min_verion_works() {
    snapshot_with_tokens! {
        pub trait ECalls {
            #[xcall(id=1, since="1.0")]
            fn foo(&self);
        }
    }
}

#[test]
fn test_invalid_since() {
    snapshot_with_tokens! {
        pub trait ECalls {
            #[xcall(id=1, since="1.0.1")]
            fn foo(&self);
        }
    }
    snapshot_with_tokens! {
        pub trait ECalls {
            #[xcall(id=1, since=1.0)]
            fn foo(&self);
        }
    }
}

#[test]
fn test_unknown_attr() {
    snapshot_with_tokens! {
        pub trait ECalls {
            #[xcall(id=1, foo="1.0.1")]
            fn foo(&self);
        }
    }
}

#[test]
fn test_not_ident_args() {
    snapshot_with_tokens! {
        pub trait ECalls {
            #[xcall(id=1)]
            fn foo(&self, Foo { bar }: Foo);
        }
    }
}

#[test]
fn test_dup_id() {
    snapshot_with_tokens! {
        pub trait ECalls {
            #[xcall(id=1)]
            fn foo(&self);
            #[xcall(id=1)]
            fn bar(&self);
        }
    }
}

#[test]
fn test_multiple_mut_ref_args() {
    snapshot_with_tokens! {
        pub trait ECalls {
            #[xcall(id=1)]
            fn foo(&self, a: &mut u32, b: &mut u32);
        }
    }
}

#[test]
fn test_no_self() {
    snapshot_with_tokens! {
        pub trait ECalls {
            #[xcall(id=1)]
            fn foo();
        }
    }
}

#[test]
fn test_invalid_id_number() {
    snapshot_with_tokens! {
        pub trait ECalls {
            #[xcall(id=-1)]
            fn foo();
        }
    }
}

#[test]
fn test_invalid_since_number() {
    snapshot_with_tokens! {
        pub trait ECalls {
            #[xcall(id=1, since="-1.0")]
            fn foo();
        }
    }
    snapshot_with_tokens! {
        pub trait ECalls {
            #[xcall(id=1, since="1.-0")]
            fn foo();
        }
    }
}

#[test]
fn test_invalid_trait_def() {
    snapshot_with_tokens! {
        pub traitb ECalls {
            #[xcall(id=1)]
            fn foo();
        }
    }
}
#[test]
fn test_invalid_trait_config() {
    let stream = crate::macro_xcall::patch(
        syn::parse_quote!(1.2),
        syn::parse_quote! { pub trait ECalls {} },
    );
    insta::assert_display_snapshot!(rustfmt_snippet::rustfmt_token_stream(&stream).unwrap())
}
