//! `tobytcp` is a library used when sending messages over a buffer, typically a `TcpStream`.
//!
//! It uses length-prefixing to allow the receiver to differentiate different messages

use std::{io, io::Write};

/// Includes methods for generating the `tobytcp` prefix, or attempting to decode the length of
/// the encoded data i.e. payload, from a buffer
pub mod protocol;

/// Writes the data, encoded as `tobytcp`, to the `Write`. Returns the total number of bytes
/// written, equal to `data.len() + 8`
/// 
/// **Note:** If there is an IO error, the data stream could be corrupted if you continue to write.
pub fn send(data: &[u8], w: &mut dyn Write) -> Result<usize, io::Error> {
    w.write_all(&protocol::tobytcp_prefix(data.len()))?;
    w.write_all(data)?;
    Ok(data.len() + 8)
}

#[cfg(test)]
mod lib_tests {
    use super::send;
    use super::protocol::tobytcp_length;
    use super::protocol::tobytcp_range;
    
    #[test]
    fn simple_send_test() {
        let to_send : Vec<u8> = vec![13, 58, 2, 4];
        let mut output : Vec<u8> = vec![];
        assert_eq!(12, send(&to_send, &mut output).unwrap());
        assert_eq!(to_send[0..], output[8..]);
    }

    #[test]
    fn read_what_was_sent() {
        let to_send : Vec<u8> = vec![13, 58, 2, 4];
        let mut output : Vec<u8> = vec![];
        assert_eq!(12, send(&to_send, &mut output).unwrap());
        
        let data_len = tobytcp_length(&output).unwrap() as usize;
        assert_eq!(to_send[..], output[8..data_len + 8]);
    }

    #[test]
    fn read_what_was_sent_data_range() {
        let to_send : Vec<u8> = vec![13, 58, 2, 4];
        let mut output : Vec<u8> = vec![];
        assert_eq!(12, send(&to_send, &mut output).unwrap());
        
        let range = tobytcp_range(&output).unwrap();
        assert_eq!(to_send.len(), range.end - range.start);
        assert_eq!(to_send[..], output[range]);
    }
}