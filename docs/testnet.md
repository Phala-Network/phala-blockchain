Test net
====

### Prepare

- `mkdir ~/phala`
- `mkdir ~/phala/gk`
- `mkdir ~/phala/pruntime_1`

### Build

- `git pull`
- `git submodule update --init`
- `cargo build --release`
- `cd standalone/pruntime/gramine-build && make dist PREFIX=../bin`

### Install

- `cp Workspaces/phala-blockchain/target/release/phala-node ~/phala`

- `cp Workspaces/phala-blockchain/target/release/pherry ~/phala`
