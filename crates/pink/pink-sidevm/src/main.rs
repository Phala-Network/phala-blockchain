use anyhow::Result;

use pink_sidevm::WasmRun;

#[tokio::main]
async fn main() -> Result<()> {
    let wat = br#"
    (module
      (type (;0;) (func (param i32 i32 i32 i32 i32) (result i32)))
      (type (;1;) (func (result i32)))
      (import "env" "sidevm_ocall" (func $sidevm_ocall (type 0)))
      (func $sidevm_poll (type 1) (result i32)
        (local i32)
        block  ;; label = @1
          i32.const 0
          i32.load offset=1048576
          local.tee 0
          i32.const -1
          i32.ne
          br_if 0 (;@1;)
          i32.const 0
          i32.const 1
          i32.const 10000
          i32.const 0
          i32.const 0
          i32.const 0
          call $sidevm_ocall
          local.tee 0
          i32.store offset=1048576
          local.get 0
          i32.const -1
          i32.ne
          br_if 0 (;@1;)
          i32.const 2
          return
        end
        i32.const 2
        local.get 0
        i32.const 0
        i32.const 0
        i32.const 0
        call $sidevm_ocall)
      (table (;0;) 1 1 funcref)
      (memory (;0;) 17)
      (global (;0;) (mut i32) (i32.const 1048576))
      (global (;1;) i32 (i32.const 1048580))
      (global (;2;) i32 (i32.const 1048592))
      (export "memory" (memory 0))
      (export "sidevm_poll" (func $sidevm_poll))
      (export "__data_end" (global 1))
      (export "__heap_base" (global 2))
      (data (;0;) (i32.const 1048576) "\ff\ff\ff\ff"))
    "#;

    let wasm_bytes = wasmer::wat2wasm(wat)?;
    let rv = WasmRun::run(wasm_bytes.as_ref(), 20)?.await?;
    println!("result: {}", rv);
    Ok(())
}
