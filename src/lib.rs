//! The `tobytcp` library provides the `TobyMessenger` struct used for sending bi-directional messages in a `TcpStream`.
#[macro_use]
extern crate log;
extern crate uuid;

pub mod protocol;

use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

use uuid::Uuid;

struct TobyReceiver {
    tcp_stream: TcpStream,
    stop: Arc<AtomicBool>,
    to_client_sender: Sender<Vec<u8>>,
    id: String,
}

impl TobyReceiver {
    // consumes tcp stream, sends finished messages to Sender's corresponding receiver
    fn receive_data(&mut self) {
        let mut raw_buff = Vec::new();
        let mut curr_size: Option<u64> = None;
        let mut done = false;
        loop {
            trace!("{}: looping toby receiver", self.id);
            if self.stop.load(Ordering::Relaxed) {
                debug!(
                    "{}: Was told to stop, shutting down inbound message consumer thread",
                    self.id
                );
                return;
            }

            let mut tcpbuf = [0u8; 256];

            match self.tcp_stream.read(&mut tcpbuf) {
                Ok(bytes) => {
                    if bytes > 0 {
                        done = false;
                        raw_buff.append(&mut tcpbuf[0..bytes].to_vec());
                    // TODO XXX Not sure if reading zero bytes is definitively the way forward!
                    } else {
                        if done {
                            debug!("{}: read zero bytes from tcp stream indicating client hangup, shutting down everything", self.id);
                            self.stop.store(true, Ordering::Relaxed);
                            match self.tcp_stream.shutdown(Shutdown::Both) {
                                Ok(()) => {}
                                Err(_) => trace!(
                                    "Got an error while shutting down tcp stream, doing nothing"
                                ),
                            }
                            return;
                        }
                        done = true;
                        trace!("{}: Read zero bytes, if this happens again, we will shutdown the thread.", self.id);
                        continue;
                    }
                }
                Err(e) => {
                    debug!("{}: Error waiting for data off of tcp stream, shutting down inbound message consumer thread {}", self.id, e);
                    return;
                }
            }

            curr_size = compute_curr_size(curr_size, &mut raw_buff);
            while curr_size.is_some() && raw_buff.len() >= to_usize(curr_size.unwrap()) {
                // get the data out!
                let parsed_message = raw_buff.drain(0..to_usize(curr_size.unwrap())).collect();
                raw_buff.shrink_to_fit();

                // reset the size, ugly!
                curr_size = compute_curr_size(None, &mut raw_buff);

                match self.to_client_sender.send(parsed_message) {
                    Ok(_) => {} // maybe log at debug
                    Err(e) => {
                        debug!("{}: Error sending a complete message to the client, shutting down inbound message consumer thread {}", self.id, e);
                        return;
                    }
                }
            }
        }
    }
}

/// TobyMessenger lets you send messages (in the form of `Vec<u8>`) over a [`TcpStream`](https://doc.rust-lang.org/std/net/struct.TcpStream.html)
///
/// # Example
///
/// ```no_run
/// # use std::net::TcpStream;
/// use tobytcp::TobyMessenger;
/// # let stream = TcpStream::connect("127.0.0.1:15235").unwrap();
///
/// let mut messenger = TobyMessenger::new(stream);
/// let receiver = messenger.start().unwrap();
///
/// loop {
///     let msg = receiver.recv().unwrap();
///     // echo msg back!
///     messenger.send(msg);
/// }
///
/// ```
///
// `TODO:` Make it clone/copyable, but for now use an `Arc<Mutex<>>` to make this threadsafe.
pub struct TobyMessenger {
    tcp_stream: TcpStream,
    stop: Arc<AtomicBool>,
    receiver_thread: Option<JoinHandle<()>>,
    writer: Mutex<()>, // guards that one person is writing to the TcpStream
    id: String,
}

impl TobyMessenger {
    /// Create a new `TobyMessenger`.
    pub fn new(tcp_stream: TcpStream) -> TobyMessenger {
        TobyMessenger {
            tcp_stream: tcp_stream,
            receiver_thread: None,
            stop: Arc::new(AtomicBool::new(false)),
            writer: Mutex::new(()),
            id: Uuid::new_v4().hyphenated().to_string(),
        }
    }

    /// Lets you see the id. Mostly for debugging, as we add the id to logging and thread names.
    pub fn id(&self) -> String {
        self.id.clone()
    }

    /// Use your `TobyMessenger` to send data synchronously and know the result!
    pub fn send(&self, data: Vec<u8>) -> std::io::Result<()> {
        match self.writer.lock() {
            Ok(_) => send_actual(&self.tcp_stream, data),
            Err(e) => {
                debug!("Error locking the writer! {}", e);
                panic!()
            }
        }
    }

    /// Starts all of the threads and queues necessary to do work
    ///
    /// The returned [`Receiver`] is to be used to process messages received over the [`TcpStream`].
    ///
    /// [`TcpStream`]: https://doc.rust-lang.org/std/net/struct.TcpStream.html
    /// [`Sender`]: https://doc.rust-lang.org/std/sync/mpsc/struct.Sender.html
    /// [`Receiver`]: https://doc.rust-lang.org/std/sync/mpsc/struct.Receiver.html
    pub fn start(&mut self) -> Result<(Receiver<Vec<u8>>), ()> {
        if self.receiver_thread.is_some() {
            debug!("Calling start on a TobyMessenger that has already started a thread!");
            return Err(());
        }

        let (inbound_sender, inbound_receiver) = channel();
        let stop_c = self.stop.clone();
        let mut success = true;

        let id_c = self.id.clone();
        match self.tcp_stream.try_clone() {
            Ok(stream) => {
                self.receiver_thread = Some(
                    thread::Builder::new()
                        .name(format!("toby_rec_{}", self.id).to_string())
                        .spawn(move || {
                            let mut rec = TobyReceiver {
                                tcp_stream: stream,
                                stop: stop_c,
                                to_client_sender: inbound_sender,
                                id: id_c,
                            };
                            rec.receive_data();
                        })
                        .unwrap(),
                );
            }
            Err(e) => {
                debug!("{}: Error cloning stream for consumer {}", self.id, e);
                success = false;
            }
        }

        if success {
            Ok(inbound_receiver)
        } else {
            Err(())
        }
    }

    /// Sends a signal to stop all of the threads, will not block on them actually shutting down.
    pub fn stop_nonblock(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        match self.tcp_stream.shutdown(Shutdown::Both) {
            Ok(()) => {}
            Err(_) => trace!("Got an error while shutting down tcp stream, doing nothing"),
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

fn send_actual(mut stream: &TcpStream, buf: Vec<u8>) -> std::io::Result<()> {
    stream.write_all(protocol::encode_tobytcp(buf).as_slice())
}

/// Goes from a slice of bytes to a u64.
fn bytes_to(bytes: &[u8]) -> u64 {
    let mut ret = 0u64;
    let mut i = 0; // hacky
    for byte in bytes {
        ret = ret | *byte as u64;
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
    num as usize
}

#[cfg(test)]
mod tests {
    use std::net::{TcpListener, TcpStream};
    use std::thread;

    #[test]
    fn test_send_data() {
        thread::spawn(|| {
            let listener = TcpListener::bind("127.0.0.1:8032").unwrap();

            // Echo the data right back!!
            for stream in listener.incoming() {
                let mut messenger = super::TobyMessenger::new(stream.unwrap());
                messenger.start().unwrap();
                messenger.send(vec![123, 4, 8]).unwrap();
            }
        });

        let stream = TcpStream::connect("127.0.0.1:8032").unwrap();

        let mut messenger = super::TobyMessenger::new(stream);
        let receiver = messenger.start().unwrap();

        assert_eq!(vec![123, 4, 8], receiver.recv().unwrap());
    }

    #[test]
    fn test_echo_single() {
        thread::spawn(|| {
            let listener = TcpListener::bind("127.0.0.1:8031").unwrap();

            // Echo the data right back!!
            for stream in listener.incoming() {
                let mut messenger = super::TobyMessenger::new(stream.unwrap());
                let receiver = messenger.start().unwrap();
                messenger.send(receiver.recv().unwrap()).unwrap();
            }
        });

        let stream = TcpStream::connect("127.0.0.1:8031").unwrap();

        let mut messenger = super::TobyMessenger::new(stream);
        let receiver = messenger.start().unwrap();

        let data = vec![31, 53, 74, 3, 67, 8, 4];
        messenger.send(data.clone()).unwrap();

        // The message should be echo'd back
        assert_eq!(data, receiver.recv().unwrap());
    }

    #[test]
    fn test_echo_loop() {
        thread::spawn(|| {
            let listener = TcpListener::bind("127.0.0.1:8037").unwrap();

            // Echo the data right back!!
            for stream in listener.incoming() {
                let mut messenger = super::TobyMessenger::new(stream.unwrap());
                let receiver = messenger.start().unwrap();
                loop {
                    let mut to_append : Vec<u8> = vec![8];
                    let mut reply = receiver.recv().unwrap();
                    reply.append(&mut to_append);
                    messenger.send(reply).unwrap();
                }
            }
        });

        let stream = TcpStream::connect("127.0.0.1:8037").unwrap();

        let mut messenger = super::TobyMessenger::new(stream);
        let receiver = messenger.start().unwrap();

        let mut ran = 0;
        for i in 0..100 {
            let mut data = vec![i];
            messenger.send(data.clone()).unwrap();
            let mut to_append : Vec<u8> = vec![8];
            data.append(&mut to_append);
            assert_eq!(data, receiver.recv().unwrap());
            ran+=1;
        }
        assert_eq!(100, ran);
    }
}
