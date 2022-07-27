# Sidevm-host

This crate is an example of the usage of the crate `pink-sidevm-host-runtime`. It can also be used
as a standalone interpreter to debug sidevm programs at development time.

## Load and run a sidevm program
Build the crate with `cargo build --release` or install it with `cargo install --path .`.

After installing the crate, you can load program with `sidevm-host`:

```bash
sidevm-host <sideprogram.wasm>
```

## Push messages to the sidevm program
Sidevm programs can receive messages from the host. There are three message channels:

- message: A arbitrary message can be sent from an ink contract in the the fat contract.
- sysmessage: A message sent from the fat contract host to the sidevm program to inform some system events: ink logs, ink message outputs, ink events emitted by commands.
- query: A query is sent from a external user via a worker local RPC in the fat contract system.

There are corresponding http API endpoints in `sidevm-host` to send those messages:

- /push/message
- /push/sysmessage
- /push/query
- /push/query/\<origin>

To simulate those messages, you can just post the payload to those endpoints with `curl`. For example:


```bash
# Send a message to the sidevm program
curl -d hello localhost:8003/push/message
# Send a query to the sidevm program
curl --data-binary @query_payload.bin localhost:8000/push/query/5Ca7afsGkHrQgwQRcfQ8u7MMrK55JYR3W2rV5KXzThNwu3GU
```
