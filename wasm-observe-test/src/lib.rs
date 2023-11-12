use std::{
    collections::HashMap,
    error::Error,
    io::Write,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use serde::{Deserialize, Serialize};
use wasmtime::{Caller, Linker};

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionCall {
    index: i32,
    name: String,
    start: SystemTime,
    end: SystemTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FCLogs {
    pub function_calls: Vec<FunctionCall>,
}

/// The InstrumentationContext holds the implementations
/// of the Observe Wasm host functions. As these are triggered,
/// this module captures the function calls and logs them.
struct InstrumentationContext {
    stack: Vec<FunctionCall>,
}

impl InstrumentationContext {
    fn new() -> Arc<Mutex<InstrumentationContext>> {
        Arc::new(Mutex::new(InstrumentationContext { stack: Vec::new() }))
    }

    fn enter(&mut self, func_index: i32, func_name: String) -> Result<(), Box<dyn Error>> {
        let fc = FunctionCall {
            index: func_index,
            name: func_name,
            start: SystemTime::now(),
            end: SystemTime::now(),
        };

        let mut fc_json =
            serde_json::to_string(&fc).expect("Failed to serialize FunctionCall to JSON");
        fc_json.push_str("\n");

        self.stack.push(fc);

        // Write the JSON data to a log file.
        append_to_log_file("log.txt", &fc_json).expect("Failed to write to log file");
        Ok(())
    }

    fn exit(&mut self, _func_index: i32) -> Result<(), Box<dyn Error>> {
        let mut fc = self.stack.pop().unwrap();
        fc.end = SystemTime::now();
        // Serialize the FunctionCall struct to JSON.
        let mut fc_json =
            serde_json::to_string(&fc).expect("Failed to serialize FunctionCall to JSON");
        fc_json.push_str("\n");

        // Write the JSON data to a log file.
        append_to_log_file("log.txt", &fc_json).expect("Failed to write to log file");
        Ok(())
    }
}

pub fn add_to_linker<T: 'static>(
    linker: &mut Linker<T>,
    wasm_bytes: Vec<u8>,
) -> Result<(), Box<dyn Error>> {
    // Parse the WebAssembly binary
    let module = walrus::ModuleConfig::new()
        .parse(&wasm_bytes)
        .expect("Failed to parse Wasm module");

    // Create a HashMap to store function names and indexes
    let mut function_map: HashMap<i32, String> = HashMap::new();

    // Iterate through the functions and extract names and indexes
    for (id,_) in module.funcs.iter_local() {
        let function = module.funcs.get(id);
            if let Some(name) = &function.name {
            function_map.insert(function.id().index() as i32, name.to_string());
        }
    }

    let ctx = InstrumentationContext::new();

    let enter_ctx = ctx.clone();

    linker.func_wrap(
        "",
        "instrument_enter",
        move |mut _caller: Caller<'_, T>, param: i32| {
            let index = param;
            let name = function_map.get(&index).unwrap().to_owned();

            if let Ok(mut cont) = enter_ctx.lock() {
                cont.enter(index,name).unwrap();
            }
        },
    )?;

    let exit_ctx = ctx.clone();

    linker.func_wrap(
        "",
        "instrument_exit",
        move|mut _caller: Caller<'_, T>, param: i32| {
            if let Ok(mut cont) = exit_ctx.lock() {
                cont.exit(param ).unwrap();
            }
        },
    )?;

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
