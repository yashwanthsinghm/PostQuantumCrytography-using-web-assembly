use pqc_dilithium::*;
use std::process;

fn main() {
    //Read command line arguments - input_file, signature_file and command verify or sign
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        println!("Usage: {} <command> <input_file> <signature_file>", args[0]);
        println!("Commands: sign or verify");
        process::exit(1);
    }
    let command = &args[1];
    let input_file = &args[2];
    let signature_file = &args[3];

    //Based on the command sign or verify the input file
    match command.as_str() {
        "sign" => sign_file(input_file, signature_file),
        "verify" => verify_file(input_file, signature_file),
        _ => {
            println!("Invalid command. Use 'sign' or 'verify'.");
        }
    }
}

fn sign_file(input_file: &str, signature_file: &str) {
    //read the input file
    let data = std::fs::read(input_file).unwrap();
    //generate a keypair
    let keypair = Keypair::generate();
    //public key
    let public_key = keypair.public;
    //sign the data
    let signature = keypair.sign(&data);
    //write to signature file
    std::fs::write(signature_file, signature).unwrap();
    //wirte to public key file
    std::fs::write("public_key.txt", public_key).unwrap();
}

fn verify_file(input_file: &str, signature_file: &str) {
    //read the input file
    let data = std::fs::read(input_file).unwrap();
    //read the signature file
    let signature = std::fs::read(signature_file).unwrap();
    //read the public key file
    let public_key = std::fs::read("public_key.txt").unwrap();
    //verify the signature
    match verify(&signature, &data, &public_key) {
        Ok(_) => println!("Signature verified"),
        Err(e) => match e {
            SignError::Input => println!("Invalid input"),
            SignError::Verify => println!("Invalid signature"),
        },
    }
}
