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
### RPC Definition
pRuntime provides some interfaces for external applications through pRPC, which are defined in [Protobuf documents](https://github.com/Phala-Network/prpc-protos/blob/master/pruntime_rpc.proto). The proto definition document can be obtained by HTTP GET request `/help`.
For example:
```
$ curl -s localhost:8000/help | head -n 20

syntax = "proto3";

import "google/protobuf/empty.proto";

package pruntime_rpc;

// The Phactory Runtime service definition.
service PhactoryAPI {
  // Get basic information about Phactory state.
  rpc GetInfo (google.protobuf.Empty) returns (PhactoryInfo) {}

  // Sync the parent chain header
  rpc SyncHeader (HeadersToSync) returns (SyncedTo) {}

  // Sync the parachain header
  rpc SyncParaHeader (ParaHeadersToSync) returns (SyncedTo) {}

```
### pRPC Visibility
These pRuntime interfaces are divided into two categories: internal RPC and external RPC. Internal RPC is used for block synchronization and other tasks, while external RPC is mainly used to provide contract call services. Therefore, pRuntime provides two server ports specified by `--port` and `--public-port`. All RPCs are available on the port specified by `--port`, while only external RPC services are provided on the port specified by `--public-port`. RPC visibility is shown in the table below:

| RPC Method | Visibility |
| ---------- | ---------- |
| SyncHeader | Private |
| SyncParaHeader | Private |
| SyncCombinedHeaders | Private |
| DispatchBlocks | Private |
| InitRuntime | Private |
| GetRuntimeInfo | Private |
| GetEgressMessages | Private |
| GetWorkerState | Private |
| AddEndpoint | Private |
| RefreshEndpointSigningTime | Private |
| GetEndpointInfo | Private |
| SignEndpointInfo | Private |
| DerivePhalaI2pKey | Private |
| Echo | Private |
| HandoverCreateChallenge | Private |
| HandoverStart | Private |
| HandoverAcceptChallenge | Private |
| HandoverReceive | Private |
| ConfigNetwork | Private |
| HttpFetch | Private |
| GetNetworkConfig | Private |
| LoadChainState | Private |
| Stop | Private |
| LoadStorageProof | Private |
| TakeCheckpoint | Private |
| GetInfo | Public |
| ContractQuery | Public |
| GetContractInfo | Public |
| GetClusterInfo | Public |
| UploadSidevmCode | Public |
| CalculateContractId | Public |

### pRPC Invocation
pRuntime starts an HTTP server on the aforementioned server ports, and the pRPC service is located at the path `/prpc/<method>`. When pRuntime is started, the available pRPC method list is printed in the logs:
```
[...] Methods under /prpc:
[...]     /prpc/PhactoryAPI.GetInfo
[...]     /prpc/PhactoryAPI.SyncHeader
[...]     /prpc/PhactoryAPI.SyncParaHeader
[...]     /prpc/PhactoryAPI.SyncCombinedHeaders
[...]     /prpc/PhactoryAPI.DispatchBlocks
[...]     /prpc/PhactoryAPI.InitRuntime
[...]     /prpc/PhactoryAPI.GetRuntimeInfo
...
```
#### Protobuf over HTTP Protocol
By default, pRPC uses the protobuf over HTTP protocol. The protocol uses the HTTP POST method, and the RPC parameters and returned data are carried in the HTTP request and response body in protobuf encoding. Client code for various programming languages can be generated based on the [protobuf definition file](https://github.com/Phala-Network/prpc-protos/blob/master/pruntime_rpc.proto).

#### JSON over HTTP Protocol
pRPC also supports the JSON over HTTP protocol. The protocol uses the HTTP GET or POST method. When using GET, the JSON protocol is used. When using POST, the JSON protocol can be switched by adding the `?json` parameter to the URL path or by adding `Content-Type: application/json` to the request header.

**Examples**:

Via GET:
```bash
$ curl localhost:8000/prpc/PhactoryAPI.GetInfo
```
Via POST with a query argument `json`:
```bash
$ curl -d "" localhost:8000/prpc/PhactoryAPI.GetInfo?json
```
Or via POST with the Content-Type of JSON:
```bash
$ curl -d "" -H "Content-Type: application/json" localhost:8000/prpc/PhactoryAPI.GetInfo
```

With any of the above calls, the RPC will respond in JSON format:
```json
{"initialized":false,"registered":false,"genesis_block_hash":null,"public_key":null,"ecdh_public_key":null,"headernum":0,"para_headernum":0,"blocknum":0,"state_root":"","dev_mode":false,"pending_messages":0,"score":0,"gatekeeper":{"role":0,"master_public_key":""},"version":"2.0.1","git_revision":"45d5fb4b66d842d6fdec4fe696f29abe675e389d-dirty","memory_usage":{"rust_used":510350,"rust_peak_used":510350,"total_peak_used":16637444096},"waiting_for_paraheaders":false,"system":null,"can_load_chain_state":true,"safe_mode_level":0,"current_block_time":0}
```

Input arguments can also use JSON. For example:
```bash
curl -d '{"url":"https://httpbin.org/post","method":"POST","body":[],"headers":[]}' localhost:8000/prpc/PhactoryAPI.HttpFetch?json
```
Outputs:
```json
{"status_code":200,"headers":[{"name":"date","value":"Thu, 09 Mar 2023 03:45:22 GMT"},{"name":"content-type","value":"application/json"},{"name":"content-length","value":"314"},{"name":"server","value":"gunicorn/19.9.0"},{"name":"access-control-allow-origin","value":"*"},{"name":"access-control-allow-credentials","value":"true"}],"body":[123,10,32,32,34,...]}
```
