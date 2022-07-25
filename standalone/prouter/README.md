PRouter
====

PRouter is a direct messaging solution among workers for Phala ecosystem.

It uses a customized version of i2pd at [here](https://github.com/Soptq/i2pd).

### Completed:
- [x] Integrate customized i2pd with PRouter.
- [x] Derive identity keypair through PRuntime and generate determinative i2pd identity from the retrieved keypair.
- [x] Bind Phala pubkey with i2pd identity on the blockchain for looking up.
- [x] Automatically i2pd config generating in order to expose local API endpoint.
- [x] i2pd SU3 file generating from netDB.
- [x] PRouter local server API to hide address translating process (Phala pubkey -> i2pd identity).

### How to build:
1. Wait for the upstream PR to be merged (https://github.com/Phala-Network/prpc-protos/pull/11).
2. Clone this branch, update all submodule to the latest.
3. Install dependencies: `sudo apt-get install libboost-all-dev libssl-dev zlib1g-dev miniupnpc`
4. `cd standalone/prouter`
5. `cargo build --release`

A binary named `prouter` should be created in `./target/release/` after compiling.

### Usage:
* Start PRouter without PRuntime and Phala Blockchain, join Phala Network (Exposing local `:8000` endpoint by default):
```
./prouter --no-pruntime --no-pnode
```

* Start PRouter with PRuntime so that deterministic identity key can be derived, no Phala Blockchain, join Phala Network:
```
./prouter --no-pnode
```

* Start PRouter with PRuntime and Phala Blockchain, will retrieve keypair from PRuntime and bind the identity on blockchain:
```
./prouter
```

More options can be checked by `./prouter --help`.

### Test TCP connection
Firstly run a simple http server at `8000` port:
```
python -m SimpleHTTPServer 8000
```
Then run the `prouter` with the `--join-pnetwork` option, you should see outputs similar to this:
```
...
[2021-12-04T10:13:18Z INFO  prouter] PRouter is starting...
[2021-12-04T10:13:18Z INFO  prouter] PRouter is successfully started
[2021-12-04T10:13:18Z INFO  prouter] Press CTRL-C to gracefully shutdown
[2021-12-04T10:13:18Z INFO  prouter]  
[2021-12-04T10:13:18Z INFO  prouter] 📦 Client Tunnels Count: 0, Server Tunnels Count: 1
[2021-12-04T10:13:18Z INFO  prouter] 🚇 Client Tunnels:
[2021-12-04T10:13:18Z INFO  prouter] 	✅ HTTP Proxy => [YOUR_HTTP_PROXY_IDENT]
[2021-12-04T10:13:18Z INFO  prouter] 	✅ SOCKS Proxy => [YOUR_SOCKS_PROXY_IDENT]
[2021-12-04T10:13:18Z INFO  prouter] 🚇 Server Tunnels:
[2021-12-04T10:13:18Z INFO  prouter] 	✅ Phala Network <= [YOUR_PNETWORK_IDENT] : 8000
[2021-12-04T10:13:18Z INFO  prouter] 💡 Network Status: Testing
[2021-12-04T10:13:18Z INFO  prouter] 🛠 Tunnel Creation Success Rate: 0%
...
```
Copying `YOUR_PNETWORK_IDENT` to somewhere else for later usage.

Next, start another PRouter on another computer (or virtual environment). After it successfully initialized, open a new command window and use the following script to test the TCP connection between workers:
```
curl -x socks5h://127.0.0.1:4447 YOUR_PNETWORK_IDENT.b32.i2p:8000
```

### Test Websocket Connection
Install https://github.com/vi/websocat on your 2 computers. Then run `websocat -v -s 8000` on your first computer. Test the WebSocket connection on your second computer by `websocat -v ws://YOUR_PNETWORK_IDENT.b32.i2p:8000 --socks5=127.0.0.1:4447`

### Performance
Under experimental conditions where client A behind a NAT tries to send an arbitrary message to client B also behind a NAT. A regular TCP message will mostly cost 0.75 seconds, A regular UDP message and a regular WebSocket message will mostly take 0.5 seconds. That is, currently we have a TCP latency of approximately 1.5 seconds and UDP and Websocket latency of approximately under 1 seconds. Note that client A and client B are both in mainland China, where nearly all i2pd floodfill routers are banned from carriers, and therefore they have to connect to foreign routers which may have high latency at the first place. Consequently, even lower average latencies are expected over world.

### Disclaimer
This software is currently in **Alpha** stage so we do not guarantee its stability. Use it at your will and at your own risk.
