extern crate tobytcp;

use std::net::{TcpListener, TcpStream};
use tobytcp::core::Messenger;
use std::io::Write;
use std::io;

fn main() {
}

fn start_server() {
    let listener = TcpListener::bind("127.0.0.1:4444").unwrap();

    let mut messengers = Vec::new();

    // accept connections and process them serially
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Server received a connection");
                let mut messenger = Messenger::new(stream);
                messenger.start();
                messengers.push(messenger);
            }
            Err(e) => println!("Error from an incoming connection: {:?}", e)
        }
    }

    println!("starting server");

    println!("starting client");
    start_client();
}

fn start_client() {
	let mut stream = TcpStream::connect("127.0.0.1:4444").unwrap();

	loop {
		let mut input = String::new();
		match io::stdin().read_line(&mut input) {
			Ok(_) => {
				stream.write(input.trim().as_bytes()).unwrap();
				//core::send(vec![0u8; 512000000], &stream);
			}
			Err(error) => println!("error: {}", error),
		}
	}
}

fn print_message(bytes: Vec<u8>) {
    println!("Message received 🤙, {} bytes: {}", bytes.len(), String::from_utf8(bytes).unwrap());
}
