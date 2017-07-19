use std::convert::TryFrom;
use std::net::{Shutdown, TcpStream};
use std::io::{Read, Write, Error, ErrorKind};

/// send data through an existing tcp stream with this
pub fn send(data: Vec<u8>, mut stream: &TcpStream) {
    let data_len_64 = u64::try_from(data.len()).unwrap();
    data_len_64.to_le();

    // Protocol says we need 8 bytes to describe the length,
    // first send the pad bytes then get to the real bytes
    stream.write(&bytes_from(data_len_64)).unwrap();

    // Write the data! Yay!
    stream.write(data.as_slice()).unwrap();
}

/// A Processor will read from a TcpStream and call your callback with messages
pub struct Processor {
    master_buffer: Vec<u8>,
    callback: fn(Vec<u8>),
    curr_chunk_size: Option<u64>,
    tcp_stream: TcpStream,
}

impl Processor {
    /// Creates a new processor, callback function will
    /// be executed once a message has been received in
    /// its entirety.
    ///
    /// The callback function will be BLOCKING as of now
    pub fn new(tcp_stream: TcpStream, callback: fn(Vec<u8>)) -> Processor {
        Processor {
            master_buffer: Vec::new(),
            callback: callback,
            curr_chunk_size: None,
            tcp_stream: tcp_stream,
        }
    }

    /// Listen will begin to listen on the TcpStream passed in through the
    /// constructor. This will call the callback function when an entire
    /// message is produced
    ///
    /// This is BLOCKING, so call this in a separate thread if you want
    /// your application to do anything
    pub fn listen(&mut self) {
        let mut read_bytes = 0;
        let mut zeroes_seen = 0; // super ugly
        loop {
            let mut buf = [0u8; 128];
            match self.tcp_stream.read(&mut buf) {
                Ok(bytes) => read_bytes = bytes,
                Err(e) => println!("error from reading: {:?}", e)
            }
			if read_bytes > 0 {
				self.process_data(buf[0..read_bytes].to_vec());
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

    /// Stop processing data and terminate the TcpStream. Call this to gracefully
    /// turn off a Processor
    pub fn shutdown(&mut self) {
        self.shutdown_stream();
    }

    fn process_data(&mut self, mut buffer: Vec<u8>) {
        self.master_buffer.append(&mut buffer);
        self.set_curr_chunk_size(); // tries to set curr chunk size

        // while we know how many bytes to look for, and we have at least enough
        // bytes to process a message, process messages
        while self.curr_chunk_size.is_some()
            && self.master_buffer.len() >= to_usize(self.curr_chunk_size.unwrap()) {
                (self.callback)(self.drain_message());
                self.curr_chunk_size = None; // BAD!! We need to reset it better
                self.set_curr_chunk_size();
        }
    }

    fn set_curr_chunk_size(&mut self) {
        // If 8 bytes in buffer, and length not set, set it
        if self.master_buffer.len() >= 8 && self.curr_chunk_size.is_none() {
            self.curr_chunk_size = Some(bytes_to(&self.master_buffer[0..8]));
            self.master_buffer.drain(0..8); // get rid of the length bytes now that we know the length
        }
    }

    // might copy ugh
    fn drain_message(&mut self) -> Vec<u8> {
        self.master_buffer.drain(0..to_usize(self.curr_chunk_size.unwrap())).collect()
    }

    /// shutdown_stream will swallow the case that the stream was not connected.
    /// The returned optional is of the error
    fn shutdown_stream(&mut self) -> Option<Error> {
        match self.tcp_stream.shutdown(Shutdown::Both) {
            Ok(()) => {
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

