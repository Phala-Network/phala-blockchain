# Phala parachain node

Based on [substrate-parachain-template](https://github.com/substrate-developer-hub/substrate-parachain-template.git), run with parity rococo branch as relay chain.

# Steps to run parachains collator locally

Note: 
 - make sure use correct relay chain spec, if you want run two parachain, use spec contains three validators
 - replace all chain indentity with yours

## Before start

Compile source code with command ```cargo build --release```

Currently collator can only run xcm V0 with polkadot [gav-xcmp](https://github.com/paritytech/polkadot/tree/gav-xcmp) branch. To make rococo local testnet run three validators, modify function at file ```<polkadot root>/service/src/chain_spec.rs```

```sh
fn rococo_local_testnet_genesis(wasm_binary: &[u8]) -> rococo_runtime::GenesisCo
                vec![
                        get_authority_keys_from_seed("Alice"),
                        get_authority_keys_from_seed("Bob"),
+                       get_authority_keys_from_seed("Charlie"),
                ],
```

After build, export new chain spec json file:

```sh
./target/release/polkadot build-spec --chain rococo-local --raw --disable-default-bootnode > rococo_local.json
```

## Step1: export parachain genesis and wasm data

 - export genesis data

```sh
./target/release/parachain-collator export-genesis-state --parachain-id 2000 > para-2000-genesis
./target/release/parachain-collator export-genesis-state --parachain-id 5000 > para-5000-genesis
```

 - export wasm data

```sh
./target/release/parachain-collator export-genesis-wasm > parachain-wasm
```

## Step2: run relay chain

- run Alice

```sh
./target/release/polkadot --validator --chain rococo_local.json --tmp --rpc-cors all --ws-port 9944 --port 30333 --alice 
```

Got Alice chain identity:
```12D3KooWKr7ueDHR83Vg1c25C19BVmSfNZhimdW65Qv3wmLAybtW```

 - run Bob (set Alice as bootnodes)

 ```sh
./target/release/polkadot --validator --chain rococo_local.json --tmp --rpc-cors all --ws-port 9955 --port 30334 --bob \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWKr7ueDHR83Vg1c25C19BVmSfNZhimdW65Qv3wmLAybtW
```

Got Bob chain identity
```12D3KooWBNohZoXDqwRCT6iJ5hxxCeaPEcjyVJaJycYoaDr1YhCK```

 - run Charlie (set Alice and Bob as bootnodes)

 ```sh
./target/release/polkadot --validator --chain rococo_local.json --tmp --rpc-cors all --ws-port 9966 --port 30335 --charlie \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWKr7ueDHR83Vg1c25C19BVmSfNZhimdW65Qv3wmLAybtW \
  --bootnodes /ip4/127.0.0.1/tcp/30334/p2p/12D3KooWBNohZoXDqwRCT6iJ5hxxCeaPEcjyVJaJycYoaDr1YhCK
```

Got Charlie chain identity
```12D3KooWHXAbtuFDLjpynQyVc7XyQqrG9qSWmUMTZH92LNgiDCBw```

## Step3 Run parachain collator 

Add ```RUST_LOG=debug RUST_BACKTRACE=1``` if you want see more details

 - run the first parachain collator

 ```sh
./target/release/parachain-collator \
  --tmp \
  --rpc-cors all --ws-port 9977 \
  --port 30336 \
  --parachain-id 2000 \
  --validator \
  -- \
  --chain ../polkadot/rococo_local.json \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWKr7ueDHR83Vg1c25C19BVmSfNZhimdW65Qv3wmLAybtW \
  --bootnodes /ip4/127.0.0.1/tcp/30334/p2p/12D3KooWBNohZoXDqwRCT6iJ5hxxCeaPEcjyVJaJycYoaDr1YhCK \
  --bootnodes /ip4/127.0.0.1/tcp/30335/p2p/12D3KooWHXAbtuFDLjpynQyVc7XyQqrG9qSWmUMTZH92LNgiDCBw
```

Got the first parachain identity:
```12D3KooWHGMhkSHP1zfQEs4powiDq2WPRNKyDugdD3vbE6hxLtPw```

 - run the second parachain collator (set first parachain as bootnodes)

 ```sh
./target/release/parachain-collator \
  --bootnodes /ip4/127.0.0.1/tcp/30336/p2p/12D3KooWDiBGAPj5VKuXQgHmb1KJiAiCmYbHvMUqoCtteEu9TudB \
  --tmp \
  --rpc-cors all --ws-port 9988 \
  --port 30337 \
  --parachain-id 5000 \
  --validator \
  -- \
  --chain ../polkadot/rococo_local.json \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWKr7ueDHR83Vg1c25C19BVmSfNZhimdW65Qv3wmLAybtW \
  --bootnodes /ip4/127.0.0.1/tcp/30334/p2p/12D3KooWBNohZoXDqwRCT6iJ5hxxCeaPEcjyVJaJycYoaDr1YhCK \
  --bootnodes /ip4/127.0.0.1/tcp/30335/p2p/12D3KooWHXAbtuFDLjpynQyVc7XyQqrG9qSWmUMTZH92LNgiDCBw
```

Got the second parachain identity (used if you want to run more parachains):
```12D3KooWCC2GkjNeF9GQH64SfC7EJwD7rZhCJ4hxU8zCaZTJ8aD3```

One last thing, following [this workshop link](https://substrate.dev/cumulus-workshop/#/3-parachains/2-register) to register parachain into relaychain, then you would see parachain begin to sync

## Step4 register custom types

At web UI, browser into ```settings/developer```, paste following json into the blank and press Save botton

```sh
{
  "ChainId": {
    "_enum": {
      "RelayChain": "Null",
      "ParaChain": "ParaId"
    }
  },
  "XCurrencyId": {
    "chain_id": "ChainId",
    "currency_id": "String"
  },
  "CurrencyId": "String",
  "EncodedXCurrencyId": "String",
  "XcmError": {
    "_enum": {
    	"Undefined": "Null",
      "Unimplemented": "Null",
      "UnhandledXcmVersion": "Null",
      "UnhandledXcmMessage": "Null",
      "UnhandledEffect": "Null",
      "EscalationOfPrivilege": "Null",
      "UntrustedReserveLocation": "Null",
      "UntrustedTeleportLocation": "Null",
      "DestinationBufferOverflow": "Null",
      "CannotReachDestination": "Null",
      "MultiLocationFull": "Null",
      "FailedToDecode": "Null",
      "BadOrigin": "Null"
    }
  }
}
```
