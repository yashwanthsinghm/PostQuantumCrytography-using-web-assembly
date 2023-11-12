use std::collections::HashSet;
use walrus::ir::Value;
use walrus::{ExportItem, LocalId, InitExpr};
use walrus::{FunctionBuilder, ValType};

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let a = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("must provide the input wasm file as the first argument and second argument as destination wasm file"))?;
    let mut module = walrus::Module::from_file(&a)?;

    // Define the type of import function
    let instrument_func_type = module.types.add(&[ValType::I32], &[]);
    // Add import functions to the module
    let (instrument_enter_function_id, _) =
        module.add_import_func("", "instrument_enter", instrument_func_type);
    let (instrument_exit_function_id, _) =
        module.add_import_func("", "instrument_exit", instrument_func_type);

    // Remove Exisiting Fucntion From Export
    let exports_to_snip: HashSet<walrus::ExportId> = module
        .exports
        .iter()
        .filter_map(|e| match e.item {
            ExportItem::Function(_) => Some(e.id()),
            _ => None,
        })
        .collect();

    for e in exports_to_snip {
        module.exports.delete(e);
    }

    let function_names: Vec<String> = module
        .funcs
        .iter_local()
        .map(|(id, _)| -> String {
            let function = module.funcs.get(id);
            if let Some(name) = &function.name {
                return name.to_owned();
            } else {
                return String::from("");
            }
        })
        .collect();


    for function_name in function_names {
        let function_id = module.funcs.by_name(&function_name).unwrap();
        let function = module.funcs.get(function_id);
        let fucntion_type = module.types.get(function.ty()).to_owned();

        // Create our parameters
        let mut function_params: Vec<LocalId> = vec![];
        for param in fucntion_type.params() {
            function_params.push(module.locals.add(*param));
        }

        //Create our results
        let mut function_results = vec![];
        for result in fucntion_type.results() {
            function_results.push(module.locals.add(*result));
        }

        //Create our instrumented function
        let mut instrumented_fucntion = FunctionBuilder::new(
            &mut module.types,
            fucntion_type.params(),
            fucntion_type.results(),
        );
        //Call instrument_enter 
        instrumented_fucntion
            .name(format!("instrument_exp_{}", function_name))
            .func_body()
            .i32_const((function.id().index() + 4) as i32)
            .call(instrument_enter_function_id);

        //Call original function
        for params in function_params.clone() {
            instrumented_fucntion.func_body().local_get(params);
        }

        instrumented_fucntion.func_body().call(function_id);

        for results in function_results.clone() {
            instrumented_fucntion.func_body().local_set(results);
        }
        
        //Call instrument_exit
        instrumented_fucntion
            .func_body()
            .i32_const((function.id().index()+4) as i32)
            .call(instrument_exit_function_id);

        //Return results
        for results in function_results.clone(){
            instrumented_fucntion.func_body().local_get(results);
        }

        let instrumented_fucntion =
            instrumented_fucntion.finish(function_params, &mut module.funcs);


        //Export instrumented function
        module.exports.add(&function_name, instrumented_fucntion);

    }

     //Add Global variables and export
     let wasm_instr_version_major = module.globals.add_local(ValType::I32, false,InitExpr::Value(Value::I32(0)));
     let wasm_instr_version_minor = module.globals.add_local(ValType::I32, false,InitExpr::Value(Value::I32(3)));

     module.exports.add("wasm_instr_version_major", wasm_instr_version_major);
     module.exports.add("wasm_instr_version_minor", wasm_instr_version_minor);


    let wasm = module.emit_wasm();
    if let Some(destination) = std::env::args().nth(2) {
        std::fs::write(destination, wasm)?;
    }

    Ok(())
}
