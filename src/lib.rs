#![feature(try_from)]
/// Core functionality for sending and receiving messages over a TcpStream, speaking in
/// **tobytcp**.
///
/// To send a message, simply use the `send` function, providing a `TcpStream`
///
/// To receive messages, construct a `Processor` object and provide a TcpStream to listen on, and a
/// callback function to execute with messages, in the form of raw bytes.
pub mod core;
