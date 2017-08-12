extern crate tobytcp;

use std::net::{TcpListener, TcpStream};
use tobytcp::TobyMessenger;
use std::thread;
use std::io;

fn main() {
    println!("Starting server");
    let server_h = thread::spawn(|| { start_server(); });
    println!("Starting client");
    let client_h = thread::spawn(|| { start_client(); });
    server_h.join().unwrap();
    client_h.join().unwrap();
    println!("Done");
}

fn start_server() {
    let listener = TcpListener::bind("127.0.0.1:4444").unwrap();

    // accept connections and process them serially
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("SERVER Server received a connection");
                let mut messenger = TobyMessenger::new(stream);
                let (_, receiver) = messenger.start();
                loop {
                    match receiver.recv() {
                        Ok(data) => print_message(data),
                        Err(e) => println!("SERVER Error receiving data {}", e),
                    }
                }
            }
            Err(e) => println!("SERVER Error from an incoming connection: {:?}", e),
        }
    }
}

fn start_client() {
    let stream = TcpStream::connect("127.0.0.1:4444").unwrap();
    let mut messenger = TobyMessenger::new(stream);
    let (sender, _) = messenger.start();

    println!("CLIENT connected to stream");
    loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                sender.send(input.trim().as_bytes().to_vec()).unwrap();
            }
            Err(error) => println!("CLIENT error: {}", error),
        }
    }
}

fn print_message(bytes: Vec<u8>) {
    println!("SERVER Message received 🤙, {}", bytes.len())
}
