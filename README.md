# `tobytcp`

This package contains the `tobytcp::TobyMessenger` struct that provides the ability to talk **TobyTcp** over a `TcpStream`.

TobyTcp is a protocol that allows for the use of a raw `tcp` stream for communicating messages, bi-directionally. See below for more details.

**NOTE**: Not ready for Production use. See below [Disclaimer](#disclaimer) section.

# [Documentation](https://docs.rs/tobytcp)

## TobyMessenger
To use a TobyMessenger to send messages over a `TcpStream`, you first create a new TobyMessenger, then `start()` it! You will get back a [Receiver](https://doc.rust-lang.org/std/sync/mpsc/struct.Receiver.html) to poll messages from. To send data, Use your TobyMessenger and call `.send()`.

```rust
extern crate tobytcp;

use std::net::TcpStream;
use tobytcp::TobyMessenger;

...

thread::spawn(|| {
    let listener = TcpListener::bind("127.0.0.1:8031").unwrap();

    // Echo the data right back!!
    for stream in listener.incoming() {
        let mut messenger = TobyMessenger::new(stream.unwrap());
        let receiver = messenger.start().unwrap();
        loop {
            receiver.recv().unwrap();
            messenger.send("Hello back!!".as_bytes().to_vec()).unwrap();
        }
    }
});

let stream = TcpStream::connect("127.0.0.1:8031").unwrap();

let mut messenger = TobyMessenger::new(stream);
let receiver = messenger.start().unwrap();

messenger.send("Hello!".as_bytes().to_vec()).unwrap();

assert_eq!("Hello back!!".as_bytes().to_vec(), receiver.recv().unwrap());
```

Version `0.10.0` made it so you have one `Receiver` returned from calling `toby_messenger.start()`, which you use to poll for messages. For sending messages, you must your `TobyMessenger`, and call `.send(..)` with your data. 

If you have to have multiple threads send data, I would arc/mutex it like such: `Arc::new(Mutex::new(toby_messenger))`, and give each thread a copy.

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
This little library has tests around the protocol and the `TobyMessenger`, but I would not use it in production service(s) just yet. The [bugs/todos section](#known-bugs-/-todos) details the specifics.

That being said, there are plenty of tests and I am not aware of any edge cases or scenarios that it breaks during, and I'm using it in my own small project and it works without issues there!

## Known Bugs / Todos
In no particular order:
- 32 bit machines may or may not work, never tested.
- Uses `as` to cast to/from `usize`, which I believe is safe, but might not build on all platforms?
- Make it threadsafe, i.e. implement `send` and `sync`. Ideally, you could just pass around a `TobyMessenger` object anywhere and send messages as you please and trust it, but now you need to `Arc<Mutex<..>>` it.

## License
The University of Illinois/NCSA (National Center for Supercomputing Applications) Open Source License

See LICENSE file. This a permissive open source license similar to Apache-2.0.
