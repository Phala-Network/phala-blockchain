# Pink Runtime

This crate contains the runtime implementation for Phala's `ink!` smart contracts.

## Overview architecture

![Alt text](assets/graph-overview.png)

As shown in the picture above, the system is composed of two parts: the chain and the worker.
The Pink Runtime runs inside the workers. There are two paths for messages from an end-user to the Pink Runtime: via on-chain transactions or off-chain queries over RPC.

Below is the call stack of a typical query flow:
![Alt text](assets/query-flow.png)

## The dylib interface

When a message from the user arrives at the worker, either via a transaction or a query, the worker decrypts the message and passes it to the Pink Runtime via the dylib interface.
From the interface point of view, the stack looks like this:
![Alt text](assets/runtime-stack.png)

There are two layers of the interface: the low-level ABI layer and the higher-level ecalls/ocalls layer.

### The low-level ABI layer

The types of the low-level ABI layer are defined in a C header file [`capi/src/v1/types.h`](../capi/src/v1/types.h).
The Pink Runtime exports only one public function, `__pink_runtime_init`.
The worker loads the Pink Runtime dylib using `dlopen` and calls `__pink_runtime_init` to initialize the runtime and convert the dylib handle into a singleton instance. Later on, the worker can call the exported functions of the dylib via the singleton instance.
There are two parameters for `__pink_runtime_init`:

-   `config`: the configuration of the runtime, mainly containing the ocall function pointers which are used by the runtime to call the host functions.
-   `ecalls`: the output parameter; the runtime will fill the ecalls function pointers into this struct so that the worker can call the runtime functions via these pointers.

After initialization, the worker can call the runtime functions via the ecalls function pointers where the cross-call ABI is defined as:

```c
typedef void (*output_fn_t)(void *ctx, const uint8_t *data, size_t len);
typedef void (*cross_call_fn_t)(uint32_t call_id, const uint8_t *data, size_t len, void *ctx, output_fn_t output);
```

Each call is identified by a call_id and comes with a data buffer as input and an output function that will be called by the callee to emit the output data.

The input data and output data will be used by the ecalls/ocalls layer to serialize/deserialize the data.

### The ecalls/ocalls layer

The ecalls/ocalls interfaces are defined in Rust [here](../capi/src/v1/mod.rs). In this layer, we defined a procedural macro #[cross_call] which can be used to decorate a trait definition containing the ecalls/ocalls function declarations. The macro will generate a generic implementation of the trait for the caller's side and a call dispatch function for the callee's side.

For example, if we have a trait definition like this:

```rust
#[cross_call(Impl)]
pub trait ECalls {
    fn foo(&self, p0: u8, p1: u8) -> u32;
    fn bar(&mut self, id: Hash) -> bool;
}
```

It will generate the following code:

```rust
pub trait ECalls {
    fn foo(&self, p0: u8, p1: u8) -> u32;
    fn bar(&mut self, id: Hash) -> bool;
}
impl<T: Impl + CrossCallMut> ECalls for T {
    fn foo(&self, p0: u8, p1: u8) -> u32 {
        let inputs = (p0, p1);
        let ret = self.cross_call(1, &inputs.encode());
        Decode::decode(&mut &ret[..]).expect("Decode failed")
    }
    fn bar(&mut self, id: Hash) -> bool {
        let inputs = (id);
        let ret = self.cross_call_mut(2, &inputs.encode());
        Decode::decode(&mut &ret[..]).expect("Decode failed")
    }
}
pub trait ECallsRo {
    fn foo(&self, p0: u8, p1: u8) -> u32;
}
impl<T: Impl> ECallsRo for T {
    fn foo(&self, p0: u8, p1: u8) -> u32 {
        let inputs = (p0, p1);
        let ret = self.cross_call(1, &inputs.encode());
        Decode::decode(&mut &ret[..]).expect("Decode failed")
    }
}
pub fn dispatch(
    executor: &mut (impl Executing + ?Sized),
    srv: &mut (impl ECalls + ?Sized),
    id: u32,
    input: &[u8],
) -> Vec<u8> {
    match id {
        1 => {
            let (p0, p1) = Decode::decode(&mut &input[..]).expect("Failed to decode args");
            executor.execute(move || srv.foo(p0, p1)).encode()
        }
        2 => {
            let (id) = Decode::decode(&mut &input[..]).expect("Failed to decode args");
            executor.execute_mut(move || srv.bar(id)).encode()
        }
        _ => panic!("Unknown call id {}", id),
    }
}
```

### Mutability of ecalls

The first parameter of each ecall function is `&self` or `&mut self`, where `&self` is used for read-only ecalls and `&mut self` is used for read-write ecalls. If state modifications are made during a read-only ecall, the changes will be discarded after the ecall returns.

## The substrate runtime

The core part of the Pink Runtime is the Substrate runtime. It is composed of pallet-contracts and a few other pallets with a chain extension where the Phala-specific features are implemented. The runtime itself is constructed at [`src/runtime.rs`](src/runtime.rs), where most of the code is standard Substrate template code. The key part is the chain extension, where the Phala-specific features are implemented at [`src/runtime/extension.rs`](src/runtime/extension.rs).
