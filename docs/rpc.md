# Phala RPC

## Full Node

### **pha_getStorageChanges**

Replay the given range of the blocks to get the accurate state trie diff, including the main storage trie and the child tries.

**Args:**

- `from`: `BlockHash` - block hash hex, the first block to replay
- `to`: `BlockHash` - block hash hex the last block to replay

**Returns:** `GetStorageChangesResponse` - the storage changes made by each block one by one from `from` to `to` (both inclusive).

To get better performance, the client should limit the amount of requested block properly. 100 blocks for each call should be OK. REQUESTS FOR TOO LARGE NUMBER OF BLOCKS WILL BE REJECTED.

### **pha_getMqNextSequence**

Return the next mq sequence number for given sender which take the ready transactions in txpool in count.

**Args:**

- `sender_hex`: `String` - the scale-codec encoded `MessageOrigin` hex of the offchain message sender to query

**Returns:** `u64` - the next expected sequence number of the sender.

## pRuntime (pRPC)

TODO
