use std::{
    io::{Read, Write},
    net::TcpStream,
};

use pqc_kyber::encapsulate;

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
    //Create a Random number generator
    let mut rng = rand::thread_rng();
    //Recieve Server's public key
    let mut server_public_key = [0u8; 1184];
    stream.read_exact(&mut server_public_key).unwrap();
    //Encapsulate a shared secret
    let (ciphertext, shared_secret_client) = encapsulate(&server_public_key, &mut rng).unwrap();
    //Send ciphertext to server
    stream.write_all(&ciphertext).unwrap();
    println!("shared_secret_bob: {:?}", shared_secret_client);
}
