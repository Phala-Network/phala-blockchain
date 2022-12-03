# Phala Blockchain

<img align="right" width="320" src="docs/static/web3 foundation_grants_badge_black.svg" alt="Funded by the web3 foundation">

![Rust](https://github.com/Phala-Network/phala-blockchain/workflows/Build/badge.svg)

Phala Network is a decentralized offchain computing protocol aiming to build the Web3 Cloud. This repo includes:

- `node/`: the main development blockchain built on Substrate
- `standalone/pherry/`: the message relayer to connect the blockchain and pRuntime
- `standalone/pruntime/`: the contract execution kernel running inside TEE enclave

## Overview

![](docs/static/phala-design.png)

The **blockchain** is the central component of the system. It records commands (confidential contract invocation), serves as the pRuntime registry, runs the native token and on-chain governance modules.

**pherry** is the message relayer. It connects the blockchain and pRuntime. It passes the block data from the chain to pRuntime and passes pRuntime side effects back to the chain. A multi-client version of the runtime bridge is being developed [here](https://github.com/Phala-Network/runtime-bridge) and now in alpha version.

**pRuntime** (Phala Network Secure Enclave Runtime) is a runtime to execute confidential smart contracts, based on confidential computing.

## Native Build

### Dependencies

<details><summary>Expand</summary>

- System dependencies
  - Ubuntu (tested with 22.04)
  ```bash
  apt install -y build-essential pkg-config libssl-dev protobuf-compiler
  ```

  - macOS
  ```bash
  brew install protobuf
  ```

  - See [here](https://grpc.io/docs/protoc-installation/) for more protobuf installation options

- Rust

  ```bash
  curl https://sh.rustup.rs -sSf | sh
  ```

- Substrate dependencies:

   ```bash
   git submodule update --init
   sh ./scripts/init.sh
   ```

- LLVM 14

  ```bash
  wget https://apt.llvm.org/llvm.sh
  chmod +x llvm.sh
  ./llvm.sh 14
  ```

</details>

### Build the blockchain and bridge

Make sure you have Rust and LLVM-10 installed.

> Note for Mac users: you also need `llvm` and `binutils` from Homebrew or MacPort, and to add their binaries to your $PATH

Run `git submodule update --init` to fetch submodules before build if you haven't add option `--recursive ` when clone code.

```bash
cargo build --release
```

The build script enforces LLVM-10 or newer is used. LLVM-10 is needed because of the wasm port of rust
crypto library, `ring`. We have to compile the C code into wasm while keeping the compatibility with
the _current_ rustc.

## Run

Please refer to [the run scripts](./scripts/run)

## Sub-pages

- [RPC](./docs/rpc.md): RPC documentations
- [Test](./docs/test.md): How to test the components

## External Resources

- [phala-wiki](https://github.com/Phala-Network/phala-wiki): The technical documentation.
- [phala-docker](https://github.com/Phala-Network/phala-docker): The production dockerfiles, including the blockchain, pherry, and pRuntime.
- [Code Bounty Program](https://forum.phala.network/t/topic/2045)
- [Responsible Disclosure](./docs/responsible-disclosure.md)
