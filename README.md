# Phala Blockchain

<img align="right" width="320" src="docs/static/web3 foundation_grants_badge_black.svg" alt="Funded by the web3 foundation">

![Rust](https://github.com/Phala-Network/phala-blockchain/workflows/Build/badge.svg)

Phala Network is a blockchain-based confidential computing cloud. This repo includes:

- `node/`: the main blockchain built on Substrate
- `standalone/pherry/`: the message relayer to connect the blockchain and pRuntime
- `standalone/pruntime/`: the contract execution kernel running inside TEE enclave

## Overview

![](docs/static/phala-design.png)

The **blockchain** is the central compoent of the system. It records commands (confidential contract invocation), serve as the pRuntime registray, runs the native token and on-chain governance modules.

**pherry** (runtime-bridge) is the message relayer. It connects the blockchain and pRuntime. It passes the block data from the chain to pRuntime and passes pRuntime side effects back to the chain. A multi-client version of the runtime bridge is being developed [here](https://github.com/Phala-Network/runtime-bridge).

**pRuntime** (Phala Network Secure Enclave Runtime) is a runtime to execute confidential smart contracts, based on confidential computing.

Related repos:

- [phala-wiki](https://github.com/Phala-Network/phala-wiki): The technical documentations.
- [apps-ng](https://github.com/Phala-Network/apps-ng): The fontend, with the UI of the Phase Wallet and the Phala confidential contract api sdk. (Will be upgraded to [apps-nng](https://github.com/Phala-Network/apps-nng) soon.)
- [phala-docker](https://github.com/Phala-Network/phala-docker): The production dockerfiles, including the blockchain, pherry, and pRuntime. 

### File structure

```text
.
├── LICENSE
├── README.md
├── pallets
│   └── phala                 Phala pallet
├── ring                      Patched ring with wasm support
├── scripts
│   ├── console.sh            Helper script to build & run the blockchain
│   └── init.sh
└───standalone
    ├── node                  Blockchain node
    ├── pruntime              pRuntime, the Secure Encalve kernel
    ├── pherry                The message relayer to connect the blockchain and pRuntime
    └── runtime               Phala Substrate Runtime
```

## Docker build

Plase refer to [phala-docker](https://github.com/Phala-Network/phala-docker).

## Native Build

### Dependencies

<details><summary>Expand</summary>

- Rust

  ```bash
  curl https://sh.rustup.rs -sSf | sh
  ```

- Substrate dependecies:

   ```bash
   git submodule init
   git submodule update
   sh ./scripts/init.sh
   ```

- LLVM 10

  ```bash
  wget https://apt.llvm.org/llvm.sh
  chmod +x llvm.sh
  ./llvm.sh 10
  ```

</details>

### Build the blockchain and bridge

Make sure you have Rust and LLVM-10 installed.

> Note for Mac users: you also need `llvm` and `binutils` from Homebrew or MacPort, and to add their binaries to your $PATH

```bash
cargo build --release
```

The build script enforces LLVM-10 or newer is used. LLVM-10 is needed because of the wasm port of rust
crypto library, `ring`. We have to compile the C code into wasm while keeping the compatibility with
the _current_ rustc.

## Run

1. Launch two local dev nodes Alice and Bob:

    ```bash
    ./scripts/console.sh start alice
    ./scripts/console.sh start bob
    ```

    - The datadir is at `$HOME/tmp/(alice|bob)`
    - Can be purged by `./scripts/console.sh purge`
    - The WebUI can connect to Alice at port 9944.

2. Compile & launch pRuntime

    ```bash
    cd standalone/pruntime
    git submodule init
    git submodule update
    ```

    Read `docs/sgx.md`, `Install SDK` section, to determine how to install the Intel SGX PSW & SDK.
    If not using Docker, you may need the following final steps:
    ```bash
    sudo mkdir /opt/intel
    sudo ln -s /opt/sgxsdk /opt/intel/sgxsdk
    ```

    Run `make` (`SGX_MODE=SW make` for simulation mode if you don't have the hardware).

    Apply for Remote Attestation API keys at
    [Intel IAS service](https://api.portal.trustedservices.intel.com/EPID-attestation). The SPID must be linkable. Then put the hex
    key in plain text files (`spid.txt` and `key.txt`) and put them into `bin/`.

    Finally, run pRuntime:
    ```bash
    cd bin/
    ./app
    ```

3. Run pherry (node and pRuntime required):

    ```bash
    ./target/release/pherry
    ```

4. Use the WebUI

    Clone the
    [Web UI for Phala Network](https://github.com/Phala-Network/apps-ng) repository and read its documentation to build and run the WebUI.


## Run with tmuxp

You can launch the full stack (semi-automatically) by:

```bash
tmuxp load ./scripts/tmuxp/three-nodes.yaml
```

Or a 4-node testnet-poc4 setup:

```bash
CHAIN=poc4 tmuxp load ./scripts/tmuxp/four-nodes.yaml
```

[tmuxp](https://tmuxp.git-pull.com/en/latest/) is a convinient tool that can bring up a tmux session
with the preconfigured commands running in panes. To play with tmuxp, it also need a tmux installed.
