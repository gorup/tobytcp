//! `tobytcp` is a library used when sending messages over a buffer, typically a `TcpStream`.
//!
//! It uses length-prefixing to allow the receiver to differentiate different messages

use std::{io, io::Write};

/// Includes methods for generating the `tobytcp` prefix, or attempting to decode the length of
/// the encoded data i.e. payload, from a buffer
pub mod protocol;

/// Writes the data, encoded as `tobytcp`, to the `Write`. Returns the total number of bytes
/// written, equal to `data.len() + 8`
pub fn send(data: &[u8], w: &mut dyn Write) -> Result<usize, io::Error> {
    w.write_all(&protocol::tobytcp_prefix(data.len()))?;
    w.write_all(data)?;
    Ok(data.len() + 8)
}
