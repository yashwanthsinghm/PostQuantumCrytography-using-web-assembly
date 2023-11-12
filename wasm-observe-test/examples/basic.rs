extern crate wasmtime;

use std::error::Error;
use wasm_observe_test::add_to_linker;
use wasmtime::*;

fn main() -> Result<(), Box<dyn Error>> {
    // Read the WebAssembly binary file into a buffer
    let args: Vec<_> = std::env::args().skip(1).collect();
    let wasm_bytes = std::fs::read(&args[0])?;

    let engine = Engine::default();

    let module = Module::new(&engine, &wasm_bytes).unwrap();

    // For host-provided functions it's recommended to use a `Linker` which does
    // name-based resolution of functions.
    let mut linker = Linker::new(&engine);

    // Provide the observability functions to the `Linker` to be made available
    // to the instrumented guest code. These are safe to add and are a no-op
    // if guest code is uninstrumented.
    add_to_linker(&mut linker, wasm_bytes)?;

    // As above, instantiation always happens within a `Store`. This means to
    // actually instantiate with our `Linker` we'll need to create a store. Note
    // that we're also initializing the store with our custom data here too.
    //
    // Afterwards we use the `linker` to create the instance.

    let mut store = Store::new(&engine, ());
    let instance = linker.instantiate(&mut store, &module)?;

    // Like before, we can get the run function and execute it.
    let add = instance.get_typed_func::<(i32, i32), i32>(&mut store, "add")?;
    let result = add.call(&mut store, (2, 3))?;
    println!("Result: {}", result);

    let sub = instance.get_typed_func::<(i32, i32), i32>(&mut store, "sub")?;
    let result = sub.call(&mut store, (5, 3))?;
    println!("Result: {}", result);

    Ok(())
}
