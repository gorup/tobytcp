#![feature(async_await)]

//! `tobytcp` is a library used when sending messages over a buffer, typically an async `TcpStream`.
//!
//! It uses length-prefixing to allow the receiver to differentiate different messages
//!
//! # Examples
//! Here is a tiny example of what it looks like to use `TobyTcp`'s built-in `send` and `receive` fns. Also look at the `/examples` directory
//! and unit tests in the source code for concrete uses of this library.
//!
//! ```no_run
//! #![feature(async_await)]
//! # use romio::TcpStream;
//! # use tobytcp::{send, receive};
//!
//! # async fn toby() -> Result<u64, std::io::Error> { // For some reason when I do Result<(), std::io::Error> it complains a ton..
//! # let mut stream = TcpStream::connect(&"127.0.0.1:7070".parse().unwrap()).await?;
//! let mut buf = vec![1, 2, 3];
//! send(&mut buf, &mut stream).await?;
//!
//! // Pretend we're connected to an echo server..
//! let received = receive(&mut stream).await?;
//!
//! assert_eq!(buf, received);
//! # Ok(8)
//! # };
//! ```
//!

/// Includes methods for generating the `tobytcp` prefix, or attempting to decode the length of
/// the encoded data i.e. payload, from a buffer.
pub mod protocol;

use futures::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use std::io;
use std::marker::Unpin;

/// Writes the data, encoded as `tobytcp`, to the `Write`. Returns the total number of bytes
/// written, equal to `data.len() + 8`. See the `examples/` dir in the source code, or the tests
/// of this fn in the source code for some examples of this being used.
///
/// Note: Do *not* perform any IO on the `Write` outside of calling `send` or `receive`! It can corrupt the tobytcp stream
pub async fn send<'w, W>(data: &'w [u8], write: &'w mut W) -> Result<usize, io::Error>
where
    W: AsyncWrite + Unpin,
{
    let prefix = protocol::tobytcp_prefix(data.len());
    write.write_all(&prefix).await?;
    write.write_all(data).await?;
    Ok(data.len() + 8)
}

/// Wait for data, which was encoded as `tobytcp`, to be received from this `Read`. Returns the data or any error.
/// See the `examples/` dir in the source code, or the tests of this fn in the source code for some examples of this being used.
///
/// Note: Do *not* perform any IO on the `Read` outside of calling `send` or `receive`! It can corrupt the tobytcp stream
pub async fn receive<R>(read: &mut R) -> Result<Vec<u8>, io::Error>
where
    R: AsyncRead + Unpin,
{
    let mut len_buf: [u8; 8] = [0; 8];
    read.read_exact(&mut len_buf).await?;

    let len = protocol::tobytcp_len(len_buf);

    let mut buf = vec![0; len as usize];
    read.read_exact(&mut buf).await?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[runtime::test]
    async fn simple_test() {
        let to_send: Vec<u8> = vec![13, 58, 2, 4];
        let mut output = Vec::new();

        let size: usize = 12;
        assert_eq!(size, send(&to_send, &mut output).await.unwrap());

        // 'manually' check the buffer
        let mut len_bytes: [u8; 8] = [0; 8];
        len_bytes.clone_from_slice(&output[0..8]);
        assert_eq!(4, u64::from_be_bytes(len_bytes));
        assert_eq!(to_send[0..], output[8..12]);

        // hacky..
        let x = output.clone();
        let mut y = x.as_slice();

        let received = receive(&mut y).await.unwrap();
        assert_eq!(received, to_send);
    }

    // Tests that if we buffer up a bunch of data that we can separate them, very important!
    #[runtime::test]
    async fn many_sends_then_receive_test() {
        let mut output = Vec::new();

        let num = 20;
        for i in 0..num {
            let to_send: Vec<u8> = vec![i, i, i, i];
            send(&to_send, &mut output).await.unwrap();
        }

        // hacky..
        let x = output.clone();
        let mut y = x.as_slice();

        for i in 0..num {
            let received = receive(&mut y).await.unwrap();
            assert_eq!(vec![i, i, i, i], received);
        }
    }
}
