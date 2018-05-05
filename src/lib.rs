#![feature(try_from)]

//! The `tobytcp` library provides the `TobyMessenger` struct used for sending bi-directional messages in a `TcpStream`.

#[macro_use]
extern crate log;
extern crate uuid;

pub mod protocol;

use std::convert::TryFrom;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use uuid::Uuid;

struct TobySender {
    tcp_stream: TcpStream,
    sender_stopped: Arc<AtomicBool>,
    from_client_receiver: Receiver<Vec<u8>>,
    timeout: Duration,
    id: String,
}

impl TobySender {
    fn send_data(&mut self) {
        loop {
            debug!("{}: looping toby sender", self.id);
            if self.sender_stopped.load(Ordering::Relaxed) {
                info!(
                    "{}: Was told to stop, shutting down outbound message consumer thread",
                    self.id
                );
                return;
            }
            match self.from_client_receiver.recv_timeout(self.timeout) {
                Ok(buf) => {
                    match send_actual(&self.tcp_stream, buf) {
                        Ok(_) => {} // maybe log at debug
                        Err(e) => error!(
                            "{}: Error sending data over tcp stream, dropped your message {}",
                            self.id, e
                        ), // TODO: Catch errors to know when to shutdown
                    }
                }
                Err(e) => {
                    match e {
                        RecvTimeoutError::Timeout => continue, // check shutdown boolean here
                        _ => {
                            info!("{}: Error waiting for messages to send from client, shutting down outbound message consumer thread", self.id);
                            return;
                        }
                    }
                }
            }
        }
    }
}

struct TobyReceiver {
    tcp_stream: TcpStream,
    receiver_stopped: Arc<AtomicBool>,
    to_client_sender: Sender<Vec<u8>>,
    id: String,
}

impl TobyReceiver {
    // consumes tcp stream, sends finished messages to Sender's corresponding receiver
    fn receive_data(&mut self) {
        let mut raw_buff = Vec::new();
        let mut curr_size: Option<u64> = None;

        loop {
            debug!("{}: looping toby receiver", self.id);
            if self.receiver_stopped.load(Ordering::Relaxed) {
                info!(
                    "{}: Was told to stop, shutting down inbound message consumer thread",
                    self.id
                );
                return;
            }

            let mut tcpbuf = [0u8; 256];
            // TODO: timeout
            match self.tcp_stream.read(&mut tcpbuf) {
                Ok(bytes) => {
                    if bytes > 0 {
                        raw_buff.append(&mut tcpbuf[0..bytes].to_vec());
                    } else {
                        continue;
                    }
                }
                Err(e) => {
                    error!("{}: Error waiting for data off of tcp stream, shutting down inbound message consumer thread {}", self.id, e);
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
                        info!("{}: Error sending a complete message to the client, shutting down inbound message consumer thread {}", self.id, e);
                        return;
                    }
                }
            }
        }
    }
}

/// TobyMessenger lets you send messages (in the form of `Vec<u8>`) over a [`TcpStream`]
/// [`TcpStream`]: https://doc.rust-lang.org/std/net/struct.TcpStream.html
///
/// # Example
///
/// ```
/// // connect to a TobyTcp server
/// use std::net::TcpStream;
/// use tobytcp::TobyMessenger;
///
/// let stream = TcpStream::connect("127.0.0.1:15235").unwrap();
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
    receiver_stopped: Arc<AtomicBool>,
    sender_stopped: Arc<AtomicBool>,
    receiver_thread: Option<JoinHandle<()>>,
    sender_thread: Option<JoinHandle<()>>,
    id: String,
}

impl TobyMessenger {
    /// Create a new `TobyMessenger`.
    pub fn new(tcp_stream: TcpStream) -> TobyMessenger {
        TobyMessenger {
            tcp_stream: tcp_stream,
            receiver_stopped: Arc::new(AtomicBool::new(true)),
            receiver_thread: None,
            sender_stopped: Arc::new(AtomicBool::new(true)),
            sender_thread: None,
            id: Uuid::new_v4().hyphenated().to_string(),
        }
    }

    /// Lets you see the id. Mostly for debugging, as we add the id to logging and thread names.
    pub fn id(&self) -> String {
        self.id.clone()
    }

    /// Sends the data, encoded as TobyTcp. As opossed to using the sender/receiver, where
    /// you will not get feedback on if the write failed.
    ///
    /// Apologies if this is awkward. You may want to create a new TobyMessenger,
    /// and just call this method on that, but to do so in a multithreaded way
    /// would require start() to be threadsafe, which I'm not sure is possible
    pub fn sync_send(tcp_stream: TcpStream, data: Vec<u8>) -> std::io::Result<()> {
        send_actual(&tcp_stream, data)
    }

    /// Use your `TobyMessenger` to send data synchronously and know the result!
    pub fn send(&self, data: Vec<u8>) -> std::io::Result<()> {
        send_actual(&self.tcp_stream, data)
    }

    /// Starts all of the threads and queues necessary to do work
    ///
    /// The returned [`Sender`] is to be used to send messages over the provided [`TcpStream`].
    ///
    /// The returned [`Receiver`] is to be used to process messages received over the [`TcpStream`].
    ///
    /// [`TcpStream`]: https://doc.rust-lang.org/std/net/struct.TcpStream.html
    /// [`Sender`]: https://doc.rust-lang.org/std/sync/mpsc/struct.Sender.html
    /// [`Receiver`]: https://doc.rust-lang.org/std/sync/mpsc/struct.Receiver.html
    pub fn start(&mut self) -> Result<(Sender<Vec<u8>>, Receiver<Vec<u8>>), ()> {
        let (inbound_sender, inbound_receiver) = channel();
        let rec_stop_c = self.receiver_stopped.clone();
        let mut success = true;

        let id_c = self.id.clone();
        match self.tcp_stream.try_clone() {
            Ok(stream) => {
                self.receiver_thread = Some(
                    thread::Builder::new()
                        .name(format!("toby_rec_{}", self.id).to_string())
                        .spawn(move || {
                            rec_stop_c.store(false, Ordering::Relaxed);
                            let mut rec = TobyReceiver {
                                tcp_stream: stream,
                                receiver_stopped: rec_stop_c,
                                to_client_sender: inbound_sender,
                                id: id_c,
                            };
                            rec.receive_data();
                        })
                        .unwrap(),
                );
            }
            Err(e) => {
                error!("{}: Error cloning stream for consumer {}", self.id, e);
                success = false;
            }
        }

        let (outbound_sender, outbound_receiver) = channel();
        let snd_stop_c = self.sender_stopped.clone();
        let id_c = self.id.clone();

        match self.tcp_stream.try_clone() {
            Ok(stream) => {
                self.sender_thread = Some(
                    thread::Builder::new()
                        .name(format!("toby_snd_{}", self.id).to_string())
                        .spawn(move || {
                            snd_stop_c.store(false, Ordering::Relaxed);
                            let mut snd = TobySender {
                                tcp_stream: stream,
                                sender_stopped: snd_stop_c,
                                from_client_receiver: outbound_receiver,
                                timeout: Duration::from_millis(100),
                                id: id_c,
                            };
                            snd.send_data();
                        })
                        .unwrap(),
                );
            }
            Err(e) => {
                error!("{}: Error cloning stream for sender {}", self.id, e);
                success = false;
            }
        }

        if success {
            Ok((outbound_sender, inbound_receiver))
        } else {
            Err(())
        }
    }

    /// Sends a signal to stop all of the threads.
    pub fn stop_nonblock(&mut self) {
        self.sender_stopped.store(true, Ordering::Relaxed);
        self.receiver_stopped.store(true, Ordering::Relaxed);
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
    num as usize
}
