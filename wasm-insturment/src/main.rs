// Import necessary libraries
use std::fs;
use regex::Regex;

fn main() {
    // Define the import instructions for WebAssembly functions
    let import_instructions = r#"
(import "dylibso_observe" "instrument_enter" (func $instrument_enter (type $t0)))
(import "dylibso_observe" "instrument_exit" (func $instrument_exit (type $t0)))
(import "dylibso_observe" "instrument_memory_grow" (func $instrument_memory_grow (type $t0)))
"#;

    // Read the original Wat file
    let wat_content = fs::read_to_string("original.wat").expect("Failed to read the Wat file");

    // Define a regular expression pattern to match the start of the first function
    let function_pattern = r#"\(func \$\w+"#;

    // Find the position to insert import instructions
    if let Some(matched) = regex::Regex::new(function_pattern)
        .unwrap()
        .find(&wat_content)
    {
        // Get the position where the import instructions should be inserted
        let insertion_position = matched.start();

        // Insert import instructions at the beginning of the Wat content
        let wat_content_with_imports = format!(
            "{}{}{}",
            &wat_content[..insertion_position],
            import_instructions,
            &wat_content[insertion_position..]
        );

        // Define a regular expression pattern to match function definitions
        let pattern = r#"\(func\s\$(?<name>\w+)\s\(export\s"(\w+)"\)\s\(type\s\$(?<type>\w+)\)\s(\(param\s\$(?<param_type_name>\w+)\s(?<param_type>\w+)\)\s)*(\(result\s(?<return_type>\w+)\))?"#;

        // Create a regex object
        let re = Regex::new(pattern).unwrap();

        // Create a vector to store WatFunction structs
        let mut functions: Vec<WatFunction> = Vec::new();

        // Iterate over function definitions in the Wat content
        for caps in re.captures_iter(&wat_content_with_imports) {
            // Extract function information
            let name = caps["name"].to_string();
            let type_name = caps["type"].to_string();
            let return_type = caps["return_type"].to_string();

            let matched_str = caps.get(0).unwrap().as_str();

            // Define a pattern to match function parameters
            let param_pattern = r#"\(param\s\$(?<param_type_name>\w+)\s(?<param_type>\w+)\)"#;

            // Create a regex object for parameters
            let param_re = Regex::new(param_pattern).unwrap();

            // Create a vector to store WatFunctionParam structs
            let mut params_list: Vec<WatFunctionParam> = Vec::new();

            // Iterate over parameter matches and capture groups
            for param_caps in param_re.captures_iter(&matched_str) {
                let param_type_name = param_caps.name("param_type_name").unwrap().as_str().to_string();
                let param_type = param_caps.name("param_type").unwrap().as_str().to_string();

                params_list.push(WatFunctionParam {
                    param_type_name,
                    param_type,
                });
            }

            // Create a WatFunction struct and add it to the functions vector
            functions.push(WatFunction {
                name,
                type_name,
                params_list,
                return_type,
            });
        }

        // Create a vector to store instrumented function strings
        let mut instrumented_functions: Vec<String> = Vec::new();

        // Iterate over the parsed functions to instrument them
        for func in functions {
            let mut instrumented_function = format!(
                r#"
            (func $instrument_exp_{0} (export "{1}") (type ${2}) "#,
                func.name, func.name, func.type_name
            );

            // Add parameter declarations to the instrumented function
            for param in &func.params_list {
                let param_string = format!(
                    r#"(param ${} {}) "#,
                    param.param_type_name, param.param_type
                );
                instrumented_function.push_str(&param_string);
            }

            // Add the return type to the instrumented function
            let result_string = format!(r#"(result {})"#, func.return_type);
            instrumented_function.push_str(&result_string);

            // Create the function body with instrumentation
            let function_body_string = format!(
                r#"
            (local $l2 {})
            (call $instrument_enter
                (i32.const 3))
            (local.set $l2
              (call ${}
            "#,
                func.return_type, func.name
            );

            instrumented_function.push_str(&function_body_string);

            // Add parameter retrieval from locals
            for param in &func.params_list {
                let param_string = format!(r#"(local.get ${})"#, param.param_type_name);
                instrumented_function.push_str(&param_string);
            }
            instrumented_function.push_str("))");

            // Add instrumentation exit code
            let function_body_end_string = format!(
                r#"
            (call $instrument_exit
                (i32.const 3))
            (local.get $l2))"#
            );

            instrumented_function.push_str(&function_body_end_string);

            // Add the instrumented function to the vector
            instrumented_functions.push(instrumented_function);
        }

        // Define a pattern to match the last function in the Wat content
        let pattern = r#"(\(func\s\$(\w+)\s\(export\s"(\w+)"\)\s\(type\s\$(\w+)\)\s\(param\s\$(\w+)\s(\w+)\)\s\(param\s\$(\w+)\s(\w+)\)\s\(result\s(\w+)\))[a-zA-Z0-9\s\(\)\$_\.]+\)\)"#;
        let re = Regex::new(pattern).unwrap();

        // Find all function matches in the Wat content
        let matches: Vec<_> = re.find_iter(&wat_content_with_imports).collect();
        let mut wat_code_with_new_functions = wat_content_with_imports.to_string();

        if let Some(last_function_match) = matches.last() {
            // Get the end position of the last function match
            let end_position = last_function_match.end();

            // Insert each new instrumented function after the last function
            for new_function in instrumented_functions {
                wat_code_with_new_functions.insert_str(end_position, &new_function);
            }
        }

        // Write the modified Wat content to a new file
        fs::write("modified.wat", wat_code_with_new_functions)
            .expect("Failed to write the modified Wat file");

        println!("Import instructions added successfully to modified.wat");
    } else {
        println!("Error: Could not find the first function in the Wat file.");
    }
}

// Define a struct to represent a WebAssembly function
#[derive(Debug)]
struct WatFunction {
    name: String,
    type_name: String,
    params_list: Vec<WatFunctionParam>,
    return_type: String,
}

// Define a struct to represent a parameter of a WebAssembly

#[derive(Debug)]
struct WatFunctionParam {
    param_type_name: String,
    param_type: String,
}

 