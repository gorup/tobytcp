#![feature(async_await)]
use futures::future::join;

use romio::{TcpListener, TcpStream};

use futures::StreamExt;
use tobytcp::{receive, send};

#[runtime::main]
async fn main() {
    join(run_bob(), run_alice()).await;
}

async fn run_bob() {
    println!("Bob starting up");

    let mut stream = loop {
        if let Ok(stream) = TcpStream::connect(&"127.0.0.1:7878".parse().unwrap()).await {
            break stream;
        }
    };

    for i in 0..10 {
        let mut msg = msg(i);
        send(&mut msg, &mut stream).await.unwrap();
        println!("Bob sent a message");

        let received = receive(&mut stream).await.unwrap();
        assert_eq!(msg, received);
        println!("Bob received a message and it matched what was sent");
    }
}

fn msg(max: u8) -> Vec<u8> {
    let mut msg = "Toby is so cute".to_string();
    for _ in 0..max {
        msg.push_str("!");
    }
    msg.into_bytes()
}

// Alice runs the listener and will echo what Bob says
async fn run_alice() {
    println!("Alice starting up");

    let mut listener = TcpListener::bind(&"127.0.0.1:7878".parse().unwrap()).unwrap();
    let mut incoming = listener.incoming();

    println!("Alice Listening on 127.0.0.1:7878");

    while let Some(stream) = incoming.next().await {
        let mut stream = stream.unwrap();
        let addr = stream.peer_addr().unwrap();

        async move {
            println!("Alice Accepting stream from: {}", addr);

            loop {
                // Will panic when Bob hangs up..
                let mut msg = receive(&mut stream).await.unwrap();
                let msg_str = String::from_utf8(msg.clone()).unwrap();
                println!("Alice received a message: {}", msg_str);
                send(&mut msg, &mut stream).await.unwrap();
            }
        }
            .await;
    }
}
