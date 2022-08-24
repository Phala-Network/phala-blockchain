# How to play
## 1. Compile and run pRuntime, pherry and phala-node.

## 2. Build

```bash
$ make
$ ls *.wasm
sideprog.wasm  start_sidevm.wasm
```

## 3. Deploy

Upload the wasm files to the chain and deploy it use js-sdk or some scripts like this:
https://gist.github.com/kvinwang/a1cfacffebbe29cf4c0d268f20b396ba


## 4. Check the instantiate status.

  We can see the following pRuntime log on instantiate success:

    [INFO  phactory::system] pink instantiate status: InstantiateStatus { nonce: [0, 1], owner: 0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48, result: Ok(ContractInfo { id: 0xcb54f9c36bc2b29c706295fadaf6864e9eb6964325b2185b6124f5645f94423d, group_id: 0x0000000000000000000000000000000000000000000000000000000000000001, pubkey: 566e290abb84f4950a89c21450ad196b2ef2acef67ce73e6a336598e167cb739 (5E22hY2d...) }) }

## 5. Start the sidevm via Query

  After the ink contract has been instantiated, we can play with it via `query` or `command`.
  We can find the ink selectors in the `target/ink/metadata.json` file.
  For example:
  ```json
  {
    "args": [],
    "docs": [],
    "label": "start_sidevm",
    "mutates": false,
    "payable": false,
    "returnType": null,
    "selector": "0x1e37db50"
  }
  ```

  Then we can invoke the `start_sidevm` function via `query`:
  
    $ ./target/release/debug-cli pink query 0x5fdb70f765123c7ab29a9c311d423e7eb0cc1a3aaaeff6702278e6ac9b9836de 0x1e37db50
