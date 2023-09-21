use std::{
    io::{Read, Write},
    net::TcpStream,
};

use pqc_kyber::encapsulate;

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
    let mut rng = rand::thread_rng();
    let mut alice_public_key = [0u8; 1184];
    stream.read_exact(&mut alice_public_key).unwrap();
    let (ciphertext, shared_secret_bob) = encapsulate(&alice_public_key, &mut rng).unwrap();
    stream.write_all(&ciphertext).unwrap();
    println!("shared_secret_bob: {:?}", shared_secret_bob);
}
