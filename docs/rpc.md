# Phala RPC

## Full Node

### **pha_getStorageChanges**

Replay the given range of the blocks to get the accurate state trie diff, including the main storage trie and the child tries.

**Args:**

- `from`: `BlockHash` - block hash hex, the first block to replay
- `to`: `BlockHash` - block hash hex the last block to replay

**Returns:** [`GetStorageChangesResponse`](https://github.com/Phala-Network/phala-blockchain/blob/ec4c8a0edcf257a4aba6d9f9ba701e3d9a382bf4/crates/phala-node-rpc-ext/src/storage_changes.rs#L67) - the storage changes made by each block one by one from `from` to `to` (both inclusive).

To get better performance, the client should limit the amount of requested block properly. 100 blocks for each call should be OK. REQUESTS FOR TOO LARGE NUMBER OF BLOCKS WILL BE REJECTED.

### **pha_getMqNextSequence**

Return the next mq sequence number for given sender which take the ready transactions in txpool in count.

**Args:**

- `sender_hex`: `String` - the scale-codec encoded [`MessageOrigin`](https://github.com/Phala-Network/phala-blockchain/blob/df3037fd85ae0e673b4b42777975c718fce8d4c8/crates/phala-mq/src/types.rs#L23) hex of the offchain message sender to query

**Returns:** `u64` - the next expected sequence number of the sender.

## pRuntime (pRPC)

TODO
