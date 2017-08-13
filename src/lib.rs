#![feature(try_from)]

//! The `tobytcp` library provides the `TobyMessenger` struct.
//!
//! by Ryan Gorup

pub mod protocol;

use std::convert::TryFrom;
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};

/// TobyMessenger lets you send messages (in the form of `Vec<u8>`) over a [`TcpStream`]
/// [`TcpStream`]: https://doc.rust-lang.org/std/net/struct.TcpStream.html
///
/// # Example
///
/// ```
/// // connect to a TobyTcp server
/// let stream = TcpStream::connect("127.0.0.1:4444").unwrap();
///
/// let mut messenger = TobyMessenger::new(stream);
/// let (sender, receiver) = messenger.start();
///
/// sender.send("Hello!".as_bytes().to_vec()).unwrap();
///
/// let recv_buf = receiver.recv().unwrap();
///
/// ```
pub struct TobyMessenger {
    tcp_stream: TcpStream,
}

impl TobyMessenger {
    /// Create a new `TobyMessenger`.
    pub fn new(tcp_stream: TcpStream) -> TobyMessenger {
        TobyMessenger { tcp_stream: tcp_stream }
    }

    /// Starts all of the threads and queues necessary to do work
    ///
    /// The returned [`Sender`] is to be used to send messages over the provided [`TcpStream`].
    ///
    /// The returned [`Receiver`] is to be used to process messages received over the [`TcpStream`].'
    ///
    /// [`TcpStream`]: https://doc.rust-lang.org/std/net/struct.TcpStream.html
    /// [`Sender`]: https://doc.rust-lang.org/std/sync/mpsc/struct.Sender.html
    /// [`Receiver`]: https://doc.rust-lang.org/std/sync/mpsc/struct.Receiver.html
    pub fn start(&mut self) -> (Sender<Vec<u8>>, Receiver<Vec<u8>>) {
        let (inbound_sender, inbound_receiver) = channel();
        match self.tcp_stream.try_clone() {
            Ok(stream) => {
                thread::spawn(move || receive_data(stream, inbound_sender));
            }
            Err(e) => println!("Error cloning stream for consumer {}", e),
        }

        let (outbound_sender, outbound_receiver) = channel();
        match self.tcp_stream.try_clone() {
            Ok(stream) => {
                thread::spawn(move || 
                    send_data(stream, outbound_receiver, Duration::from_millis(100))
                );
            }
            Err(e) => println!("Error cloning stream for sender {}", e),
        }

        return (outbound_sender, inbound_receiver);
    }
}

// consumes tcp stream, sends finished messages to Sender's corresponding receiver
fn receive_data(mut stream: TcpStream, sender: Sender<Vec<u8>>) {
    let mut raw_buff = Vec::new();
    let mut curr_size: Option<u64> = None;

    loop {
        let mut tcpbuf = [0u8; 256];
        // TODO: timeout
        match stream.read(&mut tcpbuf) {
            Ok(bytes) => {
                if bytes > 0 {
                    raw_buff.append(&mut tcpbuf[0..bytes].to_vec());
                } else {
                    continue;
                }
            }
            Err(e) => println!("Error reading from stream {}", e),
        }

        curr_size = compute_curr_size(curr_size, &mut raw_buff);
        while curr_size.is_some() && raw_buff.len() >= to_usize(curr_size.unwrap()) {
            // get the data out!
            let parsed_message = raw_buff.drain(0..to_usize(curr_size.unwrap())).collect();
            raw_buff.shrink_to_fit();

            // reset the size, ugly!
            curr_size = compute_curr_size(None, &mut raw_buff);

            match sender.send(parsed_message) {
                Ok(_) => {} // maybe log at debug
                Err(_) => {
                    println!("Error sending data to client's receiver, shutting down inbound message consumer thread");
                    return;
                }
            }
        }
    }
}

fn compute_curr_size(curr_size: Option<u64>, buf: &mut Vec<u8>) -> Option<u64> {
    if curr_size.is_none() {
        if buf.len() >= 8 {
            let size = Some(bytes_to(&buf[0..8]));
            buf.drain(0..8);
            return size;
        }
        None
    } else {
        curr_size
    }
}

fn send_data(mut stream: TcpStream, receiver: Receiver<Vec<u8>>, timeout: Duration) {
    loop {
        // check boolean here!
        match receiver.recv_timeout(timeout) {
            Ok(buf) => {
                let encoded = protocol::encode_tobytcp(buf);
                match stream.write_all(encoded.as_slice()) {
                    Ok(_) => {} // maybe log at debug
                    Err(e) => println!("Error sending data over tcp stream {}", e), // TODO: Catch errors to know when to shutdown
                }
            }
            Err(e) => {
                match e {
                    RecvTimeoutError::Timeout => continue, // check shutdown boolean here
                    RecvTimeoutError::Disconnected => {
                        println!("Error waiting for messages to send from client, shutting down outbound message consumer thread");
                        return;
                    }
                }
            }
        }
    }
}

/// Goes from a slice of bytes to a u64.
fn bytes_to(bytes: &[u8]) -> u64 {
    let mut ret = 0u64;
    let mut i = 0; // hacky
    for byte in bytes {
        ret = ret | u64::try_from(*byte).unwrap();
        if i < 7 {
            ret = ret << 8;
        }
        i = i + 1;
    }
    ret
}

/// Goes from a u64 to usize
/// TODO: This won't work for 32 bit machines, or at least it
/// wont if the value is greater than u32::MAX
fn to_usize(num: u64) -> usize {
    usize::try_from(num).unwrap()
}
