pRuntime
====

<img align="right" width="320" src="docs/static/web3 foundation_grants_badge_black.svg" alt="Funded by the web3 foundation">

**Working in progress**

This is the implementation of pRuntime (Phala Network TEE Runtime) based on Intel SGX. It's the
runtime to execute confidential smart contracts.

The runtime contains the following components:

- Identity manager
- Blockchain-pRuntime bridge
  - Remote Attestation module
  - Offchain bridge (Substrate light client)
- User-pRuntime secure communication channel
  - ECDH (secp256r1) key agreement
  - AEAD-AES-GCM-256 encryption
- Confiidential contract executor & state manager
- Restful RPC service

## Overview

```text
├── app                         Enclave loader and Restful API server (host side)
│   ├── build.rs
│   ├── src
│   │   ├── attestation.rs      Host side Remote Attestation helper
│   │   └── main.rs             Entry
├── docs
│   └── sgx.md                  SGX Hardware & SDK & Driver configuration guide
├── enclave                     pRuntime in SGX encalve
│   ├── Cargo.toml
│   ├── Makefile
│   └── src
│       ├── cert.rs             RA cert utils
│       ├── contracts           Confidential contract implementation
│       │   ├── balance.rs      Contract 2: Balances
│       │   ├── data_plaza.rs   Contract 1: DataPlaza
│       │   └── mod.rs
│       ├── cryptography
│       │   ├── aead.rs         AEAD-AES-GCM-256 encryption
│       │   ├── ecdh.rs         ECDH (secp256r1) key agreement
│       │   └── mod.rs
│       ├── hex.rs              Hex utils
│       ├── lib.rs                  pRuntime main entry
│       ├── light_validation        Substrate light client
│       │   ├── error.rs
│       │   ├── justification.rs    Block justification validation
│       │   ├── mod.rs
│       │   ├── storage_proof.rs    Storage proof validation
│       │   └── wasm_hacks.rs       Hacks to get sgx-sdk happy with Substrate
│       └── types.rs            Serializable structures
├── Makefile
├── Readme.md
└── scripts
    └── console.sh              Helper script for debugging
```

## Docker bulid

Plase refer to [plibra-grant-docker](https://github.com/Phala-Network/plibra-grant-docker). It
includes both the blockchain and pRuntime.

## Native build

### Dependencies

1. Install Intel SGX PSW and SDK as described [here](docs/sgx.md). Make sure you installed the
   correct version of SGX softwares and Rust toolchain.
2. Install Substarte dependencies (as described in
   [phala-blockchain](https://github.com/Phala-Network/phala-blockchain) readme file)
3. Apply for Remote Attestation API keys at
   [Intel IAS service](https://api.portal.trustedservices.intel.com/EPID-attestation). The SPID must be linkable. Then put the hex
   key in plain text files (`spid.txt` and `key.txt`).

### Build

To build pRuntime, the repo must be put at the same level as
[phala-blockchain](https://github.com/Phala-Network/phala-blockchain). Therefore the directory
layout should be:

```text
.
├── phala-blockchain
└── phala-pruntime
```

- Clone the repo
- Initialize Git submodules
  - `git submodule init`
  - `git submodule update`
- Run `make` (`SGX_MODE=SW make` for simulation mode if you don't have the hardware)
- Make sure put `spid.txt` (linkable) and `key.txt` into `bin/`.
- Run 
  - `cd bin`
  - `./app`

Intel SGX Driver and SDK are needed. Set environment variable `SGX_MODE=SW` while building to run
it in computer without SGX.

The dev mode keys in `spid.txt` and `key.txt` can be obtainied
[from Intel](https://software.intel.com/en-us/sgx/attestation-services).

## Send RPC to pRuntime

The repo includes a helper script `script/console.sh`. It can be used to send requests to pRuntime
directly. It's useful for testing.

```bash
./script/console.sh get_info
./script/console.sh init
./script/console.sh dump_states
```

However, in order to send commands and queries to pRuntime, it's better to use a complete setup
(blockchain, pHost (bridge), pRuntime, and frontend), because these messages are encrypted and
signed with blockchain identity and therefore not easy to be emulated by a standalone script. It's
recommended to use our Docker build for the complete setup.