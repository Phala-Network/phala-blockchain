use wasmer::{imports, wat2wasm, BaseTunables, Instance, Module, NativeFunc, Pages, Store, Function};
use wasmer_compiler_singlepass::Singlepass;
use wasmer_engine_universal::Universal;
use wasmer_tunables::LimitingTunables;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A Wasm module with one exported memory (min: 7 pages, max: unset)
    // let wat = br#"(module (memory 7) (export "memory" (memory 0)))"#;
    let wat = br#"
    (module
      (type (;0;) (func (param i32 i32) (result i32)))
      (import "env" "host_add" (func $host_add (type 0)))
      (func $add (type 0) (param i32 i32) (result i32)
        local.get 0
        local.get 1
        call $host_add)
      (table (;0;) 1 1 funcref)
      (memory (;0;) 16)
      (global (;0;) (mut i32) (i32.const 1048576))
      (global (;1;) i32 (i32.const 1048576))
      (global (;2;) i32 (i32.const 1048576))
      (export "memory" (memory 0))
      (export "add" (func $add))
      (export "__data_end" (global 1))
      (export "__heap_base" (global 2)))
    "#;

    let wasm_bytes = wat2wasm(wat)?;
    let wasm_bytes = include_bytes!("/tmp/gogo.wasm");

    // Any compiler and any engine do the job here
    let compiler = Singlepass::default();
    let engine = Universal::new(compiler).engine();

    // Here is where the fun begins

    let base = BaseTunables::for_target(&Default::default());
    let tunables = LimitingTunables::new(base, Pages(24));

    // Create a store, that holds the engine and our custom tunables
    let store = Store::new_with_tunables(&engine, tunables);

    println!("Compiling module...");
    let module = Module::new(&store, wasm_bytes)?;

    println!("Instantiating module...");
    let import_object = imports! {
      "env" => {
        "host_add" => Function::new_native(&store, |a: u32, b: u32| -> u32 { a + b + 1 }),
      }
    };

    // Now at this point, our custom tunables are used
    let instance = Instance::new(&module, &import_object)?;

    let add: NativeFunc<(u32, u32), u32> = instance.exports.get_native_function("add")?;
    let result = add.call(5, 3)?;
    println!("result: {}", result);
    Ok(())
}
