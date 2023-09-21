use std::{
    io::{Read, Write},
    net::TcpListener,
};

use pqc_kyber::{decapsulate, keypair};
fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut rng = rand::thread_rng();
        let server_keys = keypair(&mut rng).unwrap();
        stream.write_all(&server_keys.public).unwrap();
        let mut bob_ciphertext = [0u8; 1088];
        stream.read_exact(&mut bob_ciphertext).unwrap();
        let shared_secret_server = decapsulate(&bob_ciphertext, &server_keys.secret).unwrap();
        println!("shared_secret_bob: {:?}", shared_secret_server);
    }
}
