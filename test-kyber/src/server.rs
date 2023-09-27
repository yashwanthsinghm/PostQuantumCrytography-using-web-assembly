use std::{
    io::{Read, Write},
    net::TcpListener,
};

use pqc_kyber::{decapsulate, keypair};
fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        //Create a Random number generator
        let mut rng = rand::thread_rng();
        //Generate a keypair for server
        let server_keys = keypair(&mut rng).unwrap();
        //Send public key to client
        stream.write_all(&server_keys.public).unwrap();
        //Receive Client Ciphertext
        let mut ciphertext = [0u8; 1088];
        stream.read_exact(&mut ciphertext).unwrap();
        //Decapsulate shared secret
        let shared_secret_server = decapsulate(&ciphertext, &server_keys.secret).unwrap();
        println!("shared_secret_server: {:?}", shared_secret_server);
    }
}
