PR https://github.com/Phala-Network/phala-blockchain/pull/1500

Suppose we have two version of pRuntime A and B, where A is stucked, and we want to force handover to B.

SGX MR of A: 0x10c24c0e6bf8a86634417fcd8f934e62439c62907a6f1bc726906a50b054ddf10000000083d719e77deaca1470f6baf62a4d774303c899db69020f9c70ee1dfc08c7ce9e
SGX MR of B: 0xf42f7e095735702d1d3c6ac5fa3b4581d3c3673d3c5ce261a43fe782ccb3e1dc0000000083d719e77deaca1470f6baf62a4d774303c899db69020f9c70ee1dfc08c7ce9e
Genisis block hash: 0x0a15d23307d533d581291ff6dedca9ca10927c7dff6f4df9e8c3bf00bc5a6ded (Can be found in prpc::get_info)

Then the steps would be:

1. Ask at least half of the council members to sign a message as below:
```
Allow pRuntime to handover
 from: 0x10c24c0e6bf8a86634417fcd8f934e62439c62907a6f1bc726906a50b054ddf10000000083d719e77deaca1470f6baf62a4d774303c899db69020f9c70ee1dfc08c7ce9e
 to: 0xf42f7e095735702d1d3c6ac5fa3b4581d3c3673d3c5ce261a43fe782ccb3e1dc0000000083d719e77deaca1470f6baf62a4d774303c899db69020f9c70ee1dfc08c7ce9e
 genesis: 0x0a15d23307d533d581291ff6dedca9ca10927c7dff6f4df9e8c3bf00bc5a6ded
```
  See https://files.kvin.wang:8443/signit/ for an example

2. Collect the signatures and assamble them into a rpc request like this:
    ```
    $ cat sigs.json
    {
        "measurement": "f42f7e095735702d1d3c6ac5fa3b4581d3c3673d3c5ce261a43fe782ccb3e1dc0000000083d719e77deaca1470f6baf62a4d774303c899db69020f9c70ee1dfc08c7ce9e",
        "signatures": [
            {
                "signature": "fe6eeb25c088975df9bd136cc29c01a1b0bec3c4a58027efd7ca2908b233983c908a7159b81e265948a45e2c9129560f96aef24b93612f1dd4fc9aa40880ff88",
                "signature_type": 4,
                "pubkey": "d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
            },
            {
                "signature": "22591a9f308e9d1a2af2ad103334cf8ab3674a2dab9e9a6372cf1e09c8671066668ed90af1c88ad7c5c280b8e5dfb043402774cf59e38d312ee107bd8aee2f8c",
                "signature_type": 4,
                "pubkey": "8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"
            }
        ]
    }
    ```
3. Load the sigs.json to pruntime A
    ```
    $ curl -d @sigs.json  localhost:8000/prpc/PhactoryAPI.AllowHandoverTo?json
    ```
    **Note**: Don't sync any chain state to the pruntime A after this step, otherwise the handover will be rejected.
4. Run a new pruntime B instance to start the handover
    `$ ./gramine-sgx pruntime --request-handover-from http://localhost:8000`