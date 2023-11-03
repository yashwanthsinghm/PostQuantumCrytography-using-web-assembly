extern crate wasmtime;

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    error::Error,
    io::Write,
    time::SystemTime,
};
use wasmtime::*;

fn main() -> Result<(), Box<dyn Error>> {
    // Read the WebAssembly binary file into a buffer
    let wasm_bytes = std::fs::read("test.wasm").expect("Failed to read Wasm file");

    // Parse the WebAssembly binary
    let module = walrus::ModuleConfig::new()
        .parse(&wasm_bytes)
        .expect("Failed to parse Wasm module");

    // Create a HashMap to store function names and indexes
    let mut function_map: HashMap<u32, String> = HashMap::new();

    // Iterate through the functions and extract names and indexes
    for func in module.funcs.iter() {
        if let Some(name) = &func.name {
            function_map.insert(func.id().index() as u32, name.to_string());
        }
    }

    let engine = Engine::default();

    let module = Module::new(&engine, &wasm_bytes).unwrap();

    // For host-provided functions it's recommended to use a `Linker` which does
    // name-based resolution of functions.
    let mut linker = Linker::new(&engine);

    linker.func_wrap(
        "",
        "instrument_enter",
        move |mut caller: Caller<'_, FCLogs>, param: i32| {
            let index = param as u32;
            let name = function_map.get(&index).unwrap().to_owned();

            let fc = FunctionCall {
                index: param,
                name: name,
                start: SystemTime::now(),
                end: SystemTime::now(),
            };
            let mut fc_json =
                serde_json::to_string(&fc).expect("Failed to serialize FunctionCall to JSON");
            fc_json.push_str("\n");

            // Write the JSON data to a log file.
            append_to_log_file("log.txt", &fc_json).expect("Failed to write to log file");

            caller.data_mut().function_calls.push(fc);
        },
    )?;

    linker.func_wrap(
        "",
        "instrument_exit",
        |mut caller: Caller<'_, FCLogs>, _param: i32| {
            let mut fc = caller.data_mut().function_calls.pop().unwrap();
            fc.end = SystemTime::now();
            // Serialize the FunctionCall struct to JSON.
            let mut fc_json =
                serde_json::to_string(&fc).expect("Failed to serialize FunctionCall to JSON");
            fc_json.push_str("\n");

            // Write the JSON data to a log file.
            append_to_log_file("log.txt", &fc_json).expect("Failed to write to log file");
        },
    )?;

    // As above, instantiation always happens within a `Store`. This means to
    // actually instantiate with our `Linker` we'll need to create a store. Note
    // that we're also initializing the store with our custom data here too.
    //
    // Afterwards we use the `linker` to create the instance.

    let data = FCLogs {
        function_calls: Vec::new(),
    };
    let mut store = Store::new(&engine, data);
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

fn append_to_log_file(file_path: &str, data: &str) -> Result<(), std::io::Error> {
    // Open the file in append mode or create it if it doesn't exist.
    let mut file = std::fs::OpenOptions::new()
    .create(true)
    .append(true)
    .open(file_path)?;

    // Write the data to the file.
    file.write_all(data.as_bytes())?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct FunctionCall {
    index: i32,
    name: String,
    start: SystemTime,
    end: SystemTime,
}

#[derive(Debug, Serialize, Deserialize)]
struct FCLogs {
    function_calls: Vec<FunctionCall>,
}
