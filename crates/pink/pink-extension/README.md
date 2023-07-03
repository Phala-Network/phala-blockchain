<h1 align="center">Phala's ink! - Writing Enhanced Smart Contracts</h1>

Pink! is a smart contract language based on Parity's `ink!`. It extends the basic functionality with additional features, tailored to interact efficiently with Phala's Phat Contract runtime.

## Getting Started

Unaltered `ink!` contracts are fully compatible and executable on the Phala's Phat Contract platform. To learn how to start writing contracts with `ink!`, follow the Parity's [ink! documentation](https://paritytech.github.io/ink-docs/).

To get started with Pink!, add the following dependency to your `Cargo.toml`:

```toml
[dependencies]
ink = { version = "4", default-features = false }
pink = { package = "pink-extension", version = "0.4", default-features = false }

[features]
std = [
    "ink/std",
    "pink/std",
]
```

Then, you can use the `http_get!` macro to make a GET request to a remote server:

```rust
#[ink::message]
fn http_get_example(&self) {
    let response = pink::http_get!("https://httpbin.org/get");
    assert_eq!(response.status_code, 200);
}
```

## Phat Contract-Specific Features

The Pink! crate is designed to empower you, enabling you to leverage the unique features of the Phat Contract, such as making HTTP requests as demonstrated in our examples. This crate supplies the crucial types and functions needed to seamlessly interact with the Phat Contract runtime.

There are three kind of APIs to communication with the runtime:

-   Emitting Events:
    These APIs are primarily used in situations where the operation could lead to side effects that need to be deterministically recorded and may be rolled back during the execution of the contract call. For additional information on Emitting Events APIs, please refer to the [PinkEvent documentation](crate::PinkEvent).

-   Chain Extension:
    These APIs are predominantly used for read-only operations or operations that aren't expected to create deterministic side effects. For an in-depth understanding of Chain Extension APIs, please refer to the [PinkExt documentation](crate::chain_extension::PinkExtBackend).

-   System contract:
    There is a special contract called the System contract in each cluster. The system contract is instantiated when the cluster is created. Either ink contracts or external accounts can call the system contract to perform certain operations. For more information on the System contract, please refer to the [System documentation](crate::system::SystemForDoc).

For practical implementation examples, explore our [Phat Contract examples](https://github.com/Phala-Network/phat-contract-examples) repository.

## Using JavaScript with Phat Contract

Phat Contract supports JavaScript through the [phat-quickjs](https://github.com/Phala-Network/phat-quickjs) contract.

There are two ways to use JavaScript in your contract:

-   You can deploy your phat-quickjs contract instance through a standard deployment process.

-   However, for a more convenient approach, most public clusters should already have a public driver quickjs contract deployed. You can obtain the contract code_hash with `System::get_driver("JsDelegate")`.

-   For the simplest integration, consider using the [`phat_js`](https://docs.rs/phat_js/) crate. It provides an `eval` function that lets you evaluate JavaScript code snippets directly.
    For example:
    ```rust
    #[ink::message]
    fn eval_js_example(&self) {
        let result = phat_js::eval("'Hello,' + 'World'", &[]);
        assert_eq!(result, phat_js::Output::String("Hello,World".to_string()));
    }
    ```
