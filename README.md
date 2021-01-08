# Steps to run phala parachains collator locally

## Before start

Grab the Polkadot source code:

```bash
git clone https://github.com/paritytech/polkadot.git
cd polkadot
```

To make relay chain run three validators, modify function at file ```<polkadot root>/service/src/chain_spec.rs```

```sh
fn rococo_local_testnet_genesis(wasm_binary: &[u8]) -> rococo_runtime::GenesisCo
                vec![
                        get_authority_keys_from_seed("Alice"),
                        get_authority_keys_from_seed("Bob"),
+                       get_authority_keys_from_seed("Charlie"),
                ],
```

Compile source code with command ```cargo build --release --features=real-overseer```

After build, export new chain spec json file:

```sh
./target/release/polkadot build-spec --chain rococo-local --raw --disable-default-bootnode > rococo-custom.json
```

Then grab the phala blockchain source code:

```bash
git clone https://github.com/Phala-Network/phala-blockchain.git
cd phala-blockchain
git checkout rococov1
```

Compile source code with command ```cargo build --release```

## Step1: export parachain genesis and wasm data

 - export genesis data

```sh
./target/release/phala-collator export-genesis-state --chain collator --parachain-id 2000 > para-2000-genesis
./target/release/phala-collator export-genesis-state --chain collator --parachain-id 5000 > para-5000-genesis
```

 - export wasm data

```sh
./target/release/phala-collator export-genesis-wasm --chain collator > para-wasm
```

## Step2: run relay chain

- copy rococo-custom.json from polkadot directory

```sh
copy ~/polkadot/polkadot/rococo-custom.json .
```

- run Alice

```sh
./target/release/polkadot --validator --chain rococo-custom.json --tmp --node-key 0000000000000000000000000000000000000000000000000000000000000001 --rpc-cors all --ws-port 9944 --port 30333 --alice
```

Got Alice chain identity:
```12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp```

 - run Bob (set Alice as bootnodes)

 ```sh
./target/release/polkadot --validator --chain rococo-custom.json --tmp --rpc-cors all --ws-port 9955 --port 30334 --bob \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
```

 - run Charlie (set Alice as bootnodes)

 ```sh
./target/release/polkadot --validator --chain rococo-custom.json --tmp --rpc-cors all --ws-port 9966 --port 30335 --charlie \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
```

## Step3 Run phala parachain collator

Add ```RUST_LOG=debug RUST_BACKTRACE=1``` if you want see more details

 - run the first parachain collator

 ```sh
./target/release/phala-collator \
  --chain collator
  --tmp \
  --rpc-cors all \
  --ws-port 9977 \
  --port 30336 \
  --parachain-id 2000 \
  --validator \
  -- \
  --chain ../polkadot/rococo-custom.json \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
```

 - run the second parachain collator

 ```sh
./target/release/phala-collator \
  --chain collator
  --tmp \
  --rpc-cors all \
  --ws-port 9988 \
  --port 30337 \
  --parachain-id 5000 \
  --validator \
  -- \
  --chain ../polkadot/rococo-custom.json \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
```

## Step4 register custom types

At web UI, browser into ```settings/developer```, paste following json into the blank and press Save button

```sh
{
    "Id": "u32",
    "HrmpChannelId": {
      "sender": "Id",
      "recipient": "Id"
    },
    "ValidatorIndex": "u32",
    "SignedAvailabilityBitfield": {
      "payload": "AvailabilityBitfield",
      "validator_index": "ValidatorIndex",
      "signature": "ValidatorSignature",
      "real_payload": "PhantomData<AvailabilityBitfield>"
    },
    "AvailabilityBitfield": "BitVec<Lsb0, u8>",
    "SignedAvailabilityBitfields": "Vec<SignedAvailabilityBitfield>",
    "PersistedValidationData": {
      "parent_head": "HeadData",
      "block_number": "BlockNumber",
      "hrmp_mqc_heads": "Vec<(Id, Hash)>",
      "dmq_mqc_head": "Hash"
    },
    "TransientValidationData": {
      "max_code_size": "u32",
      "max_head_data_size": "u32",
      "balance": "u32",
      "code_upgrade_allowed": "Option<BlockNumber>",
      "dmq_length": "u32"
    },
    "ValidationData": {
      "persisted": "PersistedValidationData",
      "transient": "TransientValidationData"
    },
    "InboundDownwardMessage": {
        "sent_at": "BlockNumber",
        "msg": "Vec<u8>"
      },
      "InboundHrmpMessage": {
        "sent_at": "BlockNumber",
        "data": "Vec<u8>"
      },
      "MessageIngestionType": {
        "downward_messages": "Vec<InboundDownwardMessage>",
        "horizontal_messages": "BTreeMap<u32, Vec<InboundHrmpMessage>>"
      }
}
```
One last thing, to register phala parachain into relaychain, then you would see parachain begin to sync
