# `tobytcp`

This package contains the `tobytcp::TobyMessenger` struct that provides the ability to asynchronously talk **TobyTcp** over a `TcpStream`.

TobyTcp is a protocol that allows for the use of a raw `tcp` stream for communicating messages, bi-directionally. See below for more details.

## TobyMessenger
To use a TobyMessenger to send messages over a `TcpStream`, you first create a new TobyMessenger, then `start()` it to get a [Sender](https://doc.rust-lang.org/std/sync/mpsc/struct.Sender.html) and [Receiver](https://doc.rust-lang.org/std/sync/mpsc/struct.Receiver.html) that will take care of encoding the data, and sending it over the `TcpStream` asynchronously!

```rust
extern crate tobytcp;

use std::net::TcpStream;
use tobytcp::TobyMessenger;

...

let stream = TcpStream::connect("127.0.0.1:34254").unwrap();
let toby_messenger = TobyMessenger::new(stream);
let (sender, receiver) = toby_messenger.start();

// Send a 'Hello!'
sender.send("Hello!".as_bytes().to_vec()).unwrap();

// Receive a message!
let recv_buf = receiver.recv().unwrap();
```

This library is threadsafe. You can `.clone()` the `Sender` object to have multiple threads all send data through the stream, and the `TobyMessenger` will send them individually. The `Receiver` can only be owned by one thread, and there is no `.clone()` so you can only have one consumer. 

**Note:** The underlying queue mechanism is a [MultipleProducersSingleConsumer queue](https://doc.rust-lang.org/std/sync/mpsc/index.html), check out its documentation!

## TobyTcp Protocol

The TobyTcp protocol uses length prefixing for [message framing](https://blog.stephencleary.com/2009/04/message-framing.html). 

### Specification
Messages must be prefixed by eight (8) bytes, for a total of 64 bits. This 8 byte/64 bit segment of every message must contain the number of bytes present in the message being sent (NOT including the 8 bytes used for describing the size). The length prefix must be little-endian.

#### Examples
You can use the `protocol` module to encode data into TobyTcp format. No other helpers are there at this time but can be added
```
let toby_message = protocol::encode_tobytcp(my_data);
```

Here is an example of an ecnoded messages. The message has `18` bytes of data, and in the end, `18 + 8 = 26` bytes are sent, with the first 8 bytes representing the length.
```
00 00 00 00 00 00 00 12 74 6f 62 79 20 69 73 20 61 20 67 6f 6f 64 20 64 6f 67
```

## Disclaimer
This little library has not been heavily tested, and I would avoid using it in a 'production' environment. This has just been a `rust` and `tcp` learning experience for me, and it is used in a tiny project I'm working on.

Also I don't think it works on `32` bit machines..

## Known Bugs / Todos
In no particular order:
- 32 bit machines might work, but probably not if the data is over `u32::max`
- It prints to `stderr` when something breaks, which is kind of ugly
- It doesn't hang up TcpStreams properly, it just kind of stops working if the stream is broken
- Uses experimental `try_from`

## License
The University of Illinois/NCSA (National Center for Supercomputing Applications) Open Source License

See LICENSE file. This a permissive open source license similar to Apache-2.0.