//! # Examples
//! Getting the `tobytcp` length prefix that will preceed the data
//! ```
//! use tobytcp::protocol::tobytcp_prefix;
//!
//! let data = vec![1, 2, 3];
//!
//! let mut len_buf = tobytcp_prefix(data.len()); // Send `len_buf` first, then `data`
//! ```
//!
//! Decoding the length of an inbound `tobytcp` message from a stream
//! ```no_run
//! # use std::net::TcpStream;
//! # use std::io::Read;
//! use tobytcp::protocol::tobytcp_len;
//!
//! # let mut stream = TcpStream::connect("127.0.0.1:7070").unwrap();
//! let mut len_buf = [0; 8];
//! stream.read(&mut len_buf);
//!
//! let msg_len = tobytcp_len(len_buf);
//!
//! // Now we know that the next message payload is `msg_len` bytes in length..
//! ```

/// For some size, get the prefix that represents that size. Module level documentation has more info.
pub fn tobytcp_prefix(num: usize) -> [u8; 8] {
    num.to_be_bytes()
}

/// For an 8 byte buf, what is the length of the data. Module level documentation has more info.
pub fn tobytcp_len(buf: [u8; 8]) -> u64 {
    u64::from_be_bytes(buf)
}

#[cfg(test)]
mod tests {
    #[test]
    fn tobytcp_prefix_three_bytes() {
        for i in 0..16777216 {
            assert_eq!(
                [
                    0,
                    0,
                    0,
                    0,
                    0,
                    (i / 65536) as u8,
                    (i / 256) as u8,
                    (i % 256) as u8
                ],
                super::tobytcp_prefix(i as usize)
            );
        }
    }

    #[test]
    fn tobytcp_prefix_spotchecks() {
        assert_eq!([0, 0, 0, 0, 0, 0, 1, 0], super::tobytcp_prefix(256));
        assert_eq!([0, 0, 0, 0, 0, 0, 1, 1], super::tobytcp_prefix(257));
        assert_eq!([0, 0, 0, 0, 0, 0, 2, 0], super::tobytcp_prefix(512));
        assert_eq!([0, 0, 0, 0, 0, 0, 2, 89], super::tobytcp_prefix(601));
        assert_eq!([0, 0, 0, 0, 0, 0, 4, 0], super::tobytcp_prefix(1024));
        assert_eq!([0, 0, 0, 0, 0, 0, 8, 0], super::tobytcp_prefix(2048));
        assert_eq!([0, 0, 0, 0, 0, 0, 9, 9], super::tobytcp_prefix(2313));
        assert_eq!(
            [0, 0, 79, 63, 202, 21, 42, 239],
            super::tobytcp_prefix(87135391918831)
        );
    }

    #[test]
    fn tobytcp_len_three_bytes() {
        for i in 0..16777216 {
            assert_eq!(
                super::tobytcp_len([
                    0,
                    0,
                    0,
                    0,
                    0,
                    (i / 65536) as u8,
                    (i / 256) as u8,
                    (i % 256) as u8
                ]),
                i
            );
        }
    }

    #[test]
    fn tobytcp_len_spotchecks() {
        assert_eq!(super::tobytcp_len([0, 0, 0, 0, 0, 0, 1, 0]), 256);
        assert_eq!(super::tobytcp_len([0, 0, 0, 0, 0, 0, 1, 1]), 257);
        assert_eq!(super::tobytcp_len([0, 0, 0, 0, 0, 0, 2, 0]), 512);
        assert_eq!(super::tobytcp_len([0, 0, 0, 0, 0, 0, 2, 89]), 601);
        assert_eq!(super::tobytcp_len([0, 0, 0, 0, 0, 0, 4, 0]), 1024);
        assert_eq!(super::tobytcp_len([0, 0, 0, 0, 0, 0, 8, 0]), 2048);
        assert_eq!(super::tobytcp_len([0, 0, 0, 0, 0, 0, 9, 9]), 2313);
        assert_eq!(
            super::tobytcp_len([0, 0, 79, 63, 202, 21, 42, 239]),
            87135391918831
        );
    }

    #[test]
    fn tobytcp_len_and_prefix_match_three_bytes() {
        for i in 0..16777216 {
            assert_eq!(super::tobytcp_len(super::tobytcp_prefix(i as usize)), i);
        }
    }

    #[test]
    fn tobytcp_len_and_prefix_match_spotchecks() {
        assert_eq!(super::tobytcp_len(super::tobytcp_prefix(256 as usize)), 256);
        assert_eq!(
            super::tobytcp_len(super::tobytcp_prefix(132415 as usize)),
            132415
        );
        assert_eq!(
            super::tobytcp_len(super::tobytcp_prefix(3199481 as usize)),
            3199481
        );
        assert_eq!(super::tobytcp_len(super::tobytcp_prefix(3 as usize)), 3);
    }
}
