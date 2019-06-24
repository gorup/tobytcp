# `tobytcp`

TobyTcp is a protocol that allows for the use of a raw `tcp` stream for communicating messages, bi-directionally. It frames the messages with a length prefix to deliniate messages. This library provides methods to encode/decode that message prefix, but also provides `async` send & receive methods.

**NOTE**: Maybe ready for Production use. See below [Disclaimer](#disclaimer) section.

# [Documentation](https://docs.rs/tobytcp)

## Writing
Look at the `/examples` and the unit tests for compiling examples!
```rust
let prefix = protocol::tobytcp_prefix(data.len());

stream.write_all(&prefix)?;
stream.write_all(&data)?;

// OR use the send method in lib, which does almost exactly ^
send(&mut data, &mut stream).await?;
```

## Reading
Look at the `/examples` and the unit tests for compiling examples!
```rust
let mut len_buf = [0; 8];
stream.read_exact(&mut len_buf)?;
let length = protocol::tobytcp_len(len_buf);

let mut msg_buf = [0; length as usize];
stream.read_exact(&mut msg_buf)?; // Done, we have received the message into msg_buf

// OR use the receive method in lib which does almost exactly ^
let data = receive(&mut buf, &mut stream).await?;;
```

## TobyTcp Protocol

The TobyTcp protocol uses length prefixing for [message framing](https://blog.stephencleary.com/2009/04/message-framing.html).

### Specification
Messages must be prefixed by eight (8) bytes, for a total of 64 bits. This 8 byte/64 bit segment of every message must contain the number of bytes present in the message being sent (NOT including the 8 bytes used for describing the size). The length prefix must be big-endian.

#### Examples
You can use the `protocol` module to retreive the prefix, which has the length of your data. 

Here is an example of an encoded messages. The message has `18` bytes of data, and in the end, `18 + 8 = 26` bytes are sent, with the first 8 bytes representing the length.
```
00 00 00 00 00 00 00 12 74 6f 62 79 20 69 73 20 61 20 67 6f 6f 64 20 64 6f 67
```

Also see the `protocol` tests to see what is expected!

## Disclaimer
- This library provides `async` methods that are not usable with stable rust (check out [areweasyncyet](https://areweasyncyet.rs/)!), which is a large blocker for me considering this `1.0`.
  - I could put the `async` methods behind a 'feature', but I'm unsure how to do that and no-one uses this so...
- Untested on `32` bit machines, not sure it will work!

## License
The University of Illinois/NCSA (National Center for Supercomputing Applications) Open Source License

See LICENSE file. This a permissive open source license similar to Apache-2.0.
