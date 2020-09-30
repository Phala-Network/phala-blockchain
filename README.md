# Phala parachain node

Based on [substrate-parachain-template](https://github.com/substrate-developer-hub/substrate-parachain-template.git), run with parity rococo branch as relay chain.

# Steps to run parachains collator locally

Note: 
 - make sure use correct relay chain spec, if you want run two parachain, use spec contains three validators
 - replace all chain indentity with yours

## Before start

Compile source code with command ```cargo build --release```

## Step1: export parachain genesis and wasm data

 - export genesis data

```sh
./target/release/parachain-collator export-genesis-state --parachain-id 200 > para-200-genesis
./target/release/parachain-collator export-genesis-state --parachain-id 500 > para-500-genesis
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
```12D3KooWRyvvcaMhguyTTuK8MHzDRd2iZtrAtAhnTfo8V1LoLpsi```

 - run Bob (set Alice as bootnodes)

 ```sh
./target/release/polkadot --validator --chain rococo_local.json --tmp --rpc-cors all --ws-port 9955 --port 30334 --bob \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWRyvvcaMhguyTTuK8MHzDRd2iZtrAtAhnTfo8V1LoLpsi
```

Got Bob chain identity
```12D3KooWKv92bbFkFthsD21LUWHoSde5cd2GKWTnicCDr13N3vXq```

 - run Charlie (set Alice and Bob as bootnodes)

 ```sh
./target/release/polkadot --validator --chain rococo_local.json --tmp --rpc-cors all --ws-port 9966 --port 30335 --charlie \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWRyvvcaMhguyTTuK8MHzDRd2iZtrAtAhnTfo8V1LoLpsi \
  --bootnodes /ip4/127.0.0.1/tcp/30334/p2p/12D3KooWKv92bbFkFthsD21LUWHoSde5cd2GKWTnicCDr13N3vXq
```

Got Charlie chain identity
```12D3KooWD5UkeCGmsbz1oYSgc3cQdmRsiWXvHwrNmikSWsQnL5ey```

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
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWRyvvcaMhguyTTuK8MHzDRd2iZtrAtAhnTfo8V1LoLpsi \
  --bootnodes /ip4/127.0.0.1/tcp/30334/p2p/12D3KooWKv92bbFkFthsD21LUWHoSde5cd2GKWTnicCDr13N3vXq \
  --bootnodes /ip4/127.0.0.1/tcp/30335/p2p/12D3KooWD5UkeCGmsbz1oYSgc3cQdmRsiWXvHwrNmikSWsQnL5ey
```

Got the first parachain identity:
```12D3KooWAwk6koJP4LoPbVkBqvA1C87FhCmLkoWw4U7x3Hp3vcEY```

 - run the second parachain collator (set first parachain as bootnodes)

 ```sh
./target/release/parachain-collator \
  --bootnodes /ip4/127.0.0.1/tcp/30336/p2p/12D3KooWAwk6koJP4LoPbVkBqvA1C87FhCmLkoWw4U7x3Hp3vcEY \
  --tmp \
  --rpc-cors all --ws-port 9988 \
  --port 30337 \
  --parachain-id 5000 \
  --validator \
  -- \
  --chain ../polkadot/rococo_local.json \
  --bootnodes /ip4/127.0.0.1/tcp/30333/p2p/12D3KooWRyvvcaMhguyTTuK8MHzDRd2iZtrAtAhnTfo8V1LoLpsi \
  --bootnodes /ip4/127.0.0.1/tcp/30334/p2p/12D3KooWKv92bbFkFthsD21LUWHoSde5cd2GKWTnicCDr13N3vXq \
  --bootnodes /ip4/127.0.0.1/tcp/30335/p2p/12D3KooWD5UkeCGmsbz1oYSgc3cQdmRsiWXvHwrNmikSWsQnL5ey
```

Got the second parachain identity (used if you want to run more parachains):
```12D3KooWFD2W9pCp55EF78WZefxDk8XEkCXFYimVAkLysq6QU3fk```

One last thing, following [this workshop link](https://substrate.dev/cumulus-workshop/#/3-parachains/2-register) to register parachain into relaychain, then you would see parachain begin to sync
