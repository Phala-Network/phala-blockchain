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

## Scripts

- `build.sh`: Build the full node and other tools (e.g. pherry)
- `build-pruntime.sh`: Build pruntime
- `clear-pruntime.sh`: Remove pruntime checkpoints (necessary when the blockchain is reset)
- `node.sh`: Start a fresh full node in dev mode (at 19944); doesn't persist the chain db
- `pruntime.sh`: Start pruntime (at 18000)
- `pherry.sh`: Start pherry with default ports


