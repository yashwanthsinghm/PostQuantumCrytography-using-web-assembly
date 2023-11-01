use std::collections::HashSet;
use walrus::{ExportItem, Module};
use walrus::{FunctionBuilder, ValType};

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let a = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("must provide the input wasm file as the first argument"))?;
    let mut module = walrus::Module::from_file(&a)?;

    add_import_fucntions(&mut module);
    remove_existing_fucntions_from_exports(&mut module);
    add_instrumented_functions(&mut module);
    

    // Remove exports from the module
    let wasm = module.emit_wasm();
    if let Some(destination) = std::env::args().nth(2) {
        std::fs::write(destination, wasm)?;
    }
    Ok(())
}

fn add_import_fucntions(module: &mut walrus::Module) {
    // Define the type of import function
    let instrument_func_type = module.types.add(&[ValType::I32], &[]);
    // Add import functions to the module
    module.add_import_func("", "instrument_enter", instrument_func_type);
    module.add_import_func("", "instrument_exit", instrument_func_type);
}

fn remove_existing_fucntions_from_exports(module: &mut walrus::Module){
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
}

fn add_instrumented_functions(module: &mut walrus::Module){
    for fucntion in module.funcs.iter(){
        let fucntion_type = module.types.get_mut(fucntion.ty());
        let mut instrumented_fucntion  = FunctionBuilder::new(&mut module.types,fucntion_type.params(),fucntion_type.results());

        let instrumented_fucntion = instrumented_fucntion.func_body()
        .i32_const(3)
        .finish(vec![], &mut module.funcs);

       if let Some(name) = fucntion.name{
        module.exports.add(&format!("instrument_exp_{}",name),instrumented_fucntion );
       }

    }
   
}

