use std::convert::TryFrom;
use std::net::{Shutdown, TcpStream};
use std::io::{Read, Write, Error, ErrorKind};
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;

/// A `Messenger`, needs a [`TcpStream`] to send and receive data from.
///
/// [`TcpStream`]: https://doc.rust-lang.org/std/net/struct.TcpStream.html
pub struct Messenger {
    tcp_stream: TcpStream,
    send_c: (Sender<Vec<u8>>, Receiver<Vec<u8>>),
    rawdata_c: (Sender<Vec<u8>>, Receiver<Vec<u8>>),
    processed_c: (Sender<Vec<u8>>, Receiver<Vec<u8>>),
}

impl Messenger {
    /// Creates a new `Messenger`.
    pub fn new(tcp_stream: TcpStream) -> Messenger {
        Messenger {
            tcp_stream: tcp_stream,
            send_c: channel(),
            rawdata_c: channel(),
            processed_c: channel(),
        }
    }

    /// Starts all of the threads and queues necessary to do work
    pub fn start(&mut self) {
        println!("starting");

        println!("starting thread that receives from tcp");
        // maybe just give ownership of rawdata_c's sender?
        self.consume_tcp_stream();

        // DEFFFF just give ownership of rawdata_c's receiver, and processed_c's sender?
        println!("starting thread that receives raw data");
        self.consume_rawdata();

        // maybe just give ownership of send_c's receiver? needs stream
        println!("starting thread that will consume messages sent by cust");
        self.consume_sends();

        println!("done starting");
    }

    /// Send data over the [`TcpStream`] that you gave in the constructor
    ///
    /// [`TcpStream`]: https://doc.rust-lang.org/std/net/struct.TcpStream.html
    pub fn send(&mut self, mut data: Vec<u8>) {
        let data_len_64 = u64::try_from(data.len()).unwrap();
        data_len_64.to_le();

        let mut message = bytes_from(data_len_64).to_vec();
        message.append(&mut data);

        self.send_c.0.send(message);
    }

    // TODO: Somehow get receivers to cusotmer for complete messages

    /// Stop processing data and terminate the TcpStream. Call this to gracefully
    /// turn off a `Messenger` and shutdown all threads
    pub fn shutdown(&mut self) {
        self.shutdown_stream();
    }

    /// shutdown_stream will swallow the case that the stream was not connected.
    // consumes tcp stream, sends tata to rawdata_c
    fn consume_tcp_stream(&mut self) {
        let mut read_bytes = 0;
        let mut zeroes_seen = 0; // super ugly
        loop {
            let mut buf = [0u8; 128];
            match self.tcp_stream.read(&mut buf) {
                Ok(bytes) => read_bytes = bytes,
                Err(e) => println!("error from reading: {:?}", e)
            }

            if read_bytes > 0 {
                self.rawdata_c.0.send(buf.to_vec());
				zeroes_seen = 0;
			} else {
				zeroes_seen += 1;
				if zeroes_seen > 2 {
					println!("Saw {} consecutive 0 byte messages, shutting down connection", zeroes_seen);
					self.shutdown_stream();
					return;
				}
			}
        }
    }

    fn consume_rawdata(&mut self) {
        let mut buf = Vec::new();
        let mut curr_chunk_size = None;
        loop {
            match self.rawdata_c.1.recv() {
                Ok(mut bytes) => {
                    buf.append(&mut bytes);
                }
                Err(e) => println!("error consuming raw data {}", e)
            }
            // should be a method....
            if buf.len() >= 8 && curr_chunk_size.is_none() {
                curr_chunk_size = Some(bytes_to(&buf[0..8]));
                buf.drain(0..8);
            }

            while curr_chunk_size.is_some()
                && buf.len() >= to_usize(curr_chunk_size.unwrap()) {
                    let mut x = buf.drain(0..to_usize(curr_chunk_size.unwrap())).collect();
                    self.processed_c.0.send(x);
                    curr_chunk_size = None; // BAD!! We need to reset it better
                    if buf.len() >= 8 {
                        curr_chunk_size = Some(bytes_to(&buf[0..8]));
                        buf.drain(0..8);
                    }
            }
        }
    }

    fn consume_sends(&mut self) {
        loop {
            match self.send_c.1.recv() {
                Ok(message) => {
                    self.tcp_stream.write(message.as_slice()).unwrap();
                }
                Err(e) => println!("error in send_consumer {:?}", e)
            }
        }
    }

    /// The returned optional is of the error
    fn shutdown_stream(&mut self) -> Option<Error> {
        match self.tcp_stream.shutdown(Shutdown::Both) {
            Ok(_) => {
                println!("Successful shutdown");
                return None;
            }
            Err(e) => {
                if e.kind() == ErrorKind::NotConnected {
                    println!("Successful shutdown, not connected to a client");
                    return None;
                } else {
                    println!("Unsuccessful shutdown, error: {}", e);
                    return Some(e);
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

/// Goes from a single u64 to 8xu8
fn bytes_from(mut num: u64) -> [u8; 8] {
    let mut ret = [0u8; 8];

    for (i, _) in (0..7).enumerate() {
        ret[7 - i] = u8::try_from(num & 0b1111_1111_u64).unwrap();
        num = num >> 8;
    }
    ret
}


/// Goes from a u64 to usize
/// TODO: This won't work for 32 bit machines, or at least it
/// wont if the value is greater than u32::MAX
fn to_usize(num: u64) -> usize {
    usize::try_from(num).unwrap()
}

