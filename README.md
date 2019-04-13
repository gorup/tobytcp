# `tobytcp`

This package contains the `tobytcp::TobyMessenger` struct that provides the ability to talk **TobyTcp** over a `Write`, but usually a `TcpStream`.

TobyTcp is a protocol that allows for the use of a raw `tcp` stream for communicating messages, bi-directionally. See below for more details.

**NOTE**: Not ready for Production use. See below [Disclaimer](#disclaimer) section.

# [Documentation](https://docs.rs/tobytcp)

## Writing
```
let prefix = protocol::tobytcp_prefix(data.len());

stream.write_all(&prefix)?;
stream.write_all(&data)?;

// OR use the send method in lib, which does exactly ^
send(&data, stream)?;
```

## Reading
```
num_read += stream.read(&mut buf)?;
let length = protocol::tobytcp_length(&buf);

if length.is_none || num_read < length.unwrap() {
    // keep reading, we don't have enough data
    continue;
}

let data = buf[protocol::tobytcp_length_to_range(length)];
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
- Untested on `32` bit machines, not sure it will work!

## License
The University of Illinois/NCSA (National Center for Supercomputing Applications) Open Source License

See LICENSE file. This a permissive open source license similar to Apache-2.0.
