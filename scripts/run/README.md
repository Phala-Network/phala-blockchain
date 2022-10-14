# Scripts for running a local dev network

This directory contains the convenient scripts to quickly start a local dev network.

## Typical usage

Make sure you are in the root of the git repo.

Build the project:

```bash
./scripts/run/build.sh
./scripts/run/build-pruntime.sh
```

Start a dev testnet:

```bash
./scripts/run/clear-pruntime.sh
# run the following commands in 3 terminal windows or tmux panes
./scripts/run/node.sh
./scripts/run/pruntime.sh
./scripts/run/pherry.sh
```

Now you have full node at `ws://localhost:19944`, and pruntime at `http://localhost:18000`.

## Setup Phat Contract environment

In addition to a fresh local testnet setup, you will also want to set up the Phat Contract runtime. This can be done with our js script:

```bash
cd scripts/js
yarn
node src/bin/setupTestnet.js
```

Note that the script connect to the default ports of the node and pruntime. If you want to set it up differently, you may want to check the arguments of the script:

```bash
node src/bin/setupTestnet.js --help

# Usage: setupTestnet [options]
#
# Set up a bare testnet with a single worker to run Phat Contract. The worker will be the only Gatekeeper and the worker to run contracts.
#
# Options:
#   --substrate <endpoint>    substrate ws rpc endpoint (default: "ws://localhost:19944")
#   --root-key <key>          root key SURI (default: "//Alice")
#   --root-type <key-type>    root key type (default: "sr25519")
#   --pruntime <pr-endpoint>  pruntime rpc endpoint (default: "http://localhost:18000")
#   -h, --help                display help for command
```

> WIP: SideVM and logger are not currently set up by the script yet.

## Scripts

- `build.sh`: Build the full node and other tools (e.g. pherry)
- `build-pruntime.sh`: Build pruntime
- `clear-pruntime.sh`: Remove pruntime checkpoints (necessary when the blockchain is reset)
- `node.sh`: Start a fresh full node in dev mode (at 19944); doesn't persist the chain db
- `pruntime.sh`: Start pruntime (at 18000)
- `pherry.sh`: Start pherry with default ports


