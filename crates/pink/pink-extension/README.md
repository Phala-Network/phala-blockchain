<h1 align="center">
    Phala's ink! for writing fat contracts
</h1>

pink! is based on Parity's ink! language and provide some extra functionality to interact with phala's fat contract runtime.

# Getting started

The orignal `ink!` contract can run under Phala's fat contract platform without any modifacation. So you can follow Parity's `ink!` [documentation](https://paritytech.github.io/ink-docs/) to get started.

If you want to use fat contract's specific features, such as phala-mq message, HTTP requests, you can use `pink_extension` to achieve this. See [examples](https://github.com/Phala-Network/phala-blockchain/tree/master/crates/pink/examples) for more detail.
