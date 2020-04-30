# Phala Blockchain

<img align="right" width="320" src="docs/static/web3 foundation_grants_badge_black.svg" alt="Funded by the web3 foundation">

Phala Network is a TEE-Blockchain hybrid architecture implementing Confidential Contract. This repo
includes:

- `node/`: the main blockchain built on Substrate
- `phost/`: the bridge daemon to connect the blockchain and
  [pRuntime](https://github.com/Phala-Network/phala-pruntime)

## Overview

![](docs/static/diagram.png)

The **blockchain** is the central compoent of the system. It records commands (confidential contract
invocation), serve as the pRuntime registray, runs the native token and on-chain governance modules.

**pHost** is a daemon program that connects the blockchain and the pRuntime. It passes the block
data from the chain to pRuntime and passes pRuntime side effects back to the chain.

Related repos:

- [phala-docs](https://github.com/Phala-Network/phala-docs): The central repo for documentations.
- [phala-pruntime](https://github.com/Phala-Network/phala-pruntime): The cotract executor running
  inside TEE enclaves.
- [phala-polka-apps](https://github.com/Phala-Network/phala-polka-apps): The Web UI and SDK to
  interact with confidential contract. Based on polkadot.js.
- [plibra-grant-docker](https://github.com/Phala-Network/plibra-grant-docker): The W3F M2 docker
  build with the blockchain, pHost and pRuntime.

### File structure

```text
.
├── LICENSE
├── README.md
├── node                      Blockchain node
│   ├── runtime               The runtime
│   │   └── src
│   │       ├── execution.rs  Phala Network's main runtime module
│   │       └── lib.rs        Entry of the runtime
│   ├── scripts
│   │   ├── ccwrapper/        Helper scripts for building
│   │   ├── console.sh        Helper script to build & run the blockchain
│   │   └── init.sh
│   └── src/                  The node
├── phost                     The bridge deamon "pHost"
│   ├── scripts               
│   │   └── console.sh        Helper script
│   └── src
└── ring                      Patched ring with wasm support
```

## Docker bulid

Plase refer to [plibra-grant-docker](https://github.com/Phala-Network/plibra-grant-docker). It includes both the blockchain and pRuntime.

## Native Build

### Dependencies

<details><summary>Expand</summary>

- Rust

  ```bash
  curl https://sh.rustup.rs -sSf | sh
  ```

- Substrate dependecies:

   ```bash
   cd node
   sh ./scripts/init.sh
   ```

- LLVM 10

  ```bash
  wget https://apt.llvm.org/llvm.sh
  chmod +x llvm.sh
  ./llvm.sh 10
  ```

</details>

### Build the blockchain

Make sure you have Rust and LLVM-10 installed.

```bash
cd node
./scripts/console.sh wrap-build
```

The above script runs a regular `cargo build --release` and enforce LLVM-9 is used in addition.
LLVM-9 is needed because of the wasm port of rust crypto library, `ring`. We have to compile the C
code into wasm while keeping the compatibiliy with the _current_ rustc.

### Build the bridge

```bash
cd phost
./scripts/console.sh build --release
```

## Run

1. Launch two local dev nodes Alice and Bob:

    ```bash
    cd node
    ./scripts/console.sh start alice
    ./scripts/console.sh start bob
    ```

    - The datadir is at `/tmp/$USER/(alice|bob)`
    - Can be purged by `./scripts/console.sh purge`
    - The WebUI can connect to Alice at port 9944.

2. Run pHost (please start pRuntime first):

    ```bash
    cd phost
    ./target/release/phost -f "/tmp/${USER}/alice/chains/local_testnet/genesis-info.txt"
    ```

    - `-f`: Specify the genesis state to initialize the Substrate bridge in pRuntime. The file is
      produced by the blockchain node when launching.
    - pHost quits every time the blockchain node or the pRuntime is down. Remember to relauch when
      necessary.
