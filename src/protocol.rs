/// Provides the length of the data, but also the number of bytes in the
/// provided data that represent the length of the data. In short, you should
/// ignore <retval.1> bytes and process the next <retval.1> bytes as data.
pub fn decode_tobytcp2(bytes: &[u8]) -> (u64, u8) {
    let mut len: u64 = 0;
    let mut i = 0;
    for byte in bytes.iter() {
        i += 1;
        len |= *byte as u64 & 0b0111_1111_u64;
        if *byte > 127 {
            len <<= 7;
        }
    }

    (len, i)
}

/// Returns the prefix you should use to represent num_bytes using tobytcp2
pub fn tobytcp2_prefix(mut len: u64) -> Vec<u8> {
    let mut ret: Vec<u8> = Vec::new();

    ret.push(len as u8 & 0b0111_1111);
    while len > 127 {
        len >>= 7;
        ret.push(len as u8 | 0b1000_0000);
    }

    ret.reverse();
    ret
}

#[cfg(test)]
mod tobytcp_tests {
    #[test]
    fn encode_single_byte() {
        let message = vec![100, 13, 69, 17];
        let encoded = super::encode_tobytcp(message);
        // We had 4 bytes of data
        assert_eq!(vec![0, 0, 0, 0, 0, 0, 0, 4, 100, 13, 69, 17], encoded);
    }

    #[test]
    fn encode_bigger_message() {
        let data = vec![69; 257];
        let mut expected = vec![0, 0, 0, 0, 0, 0, 1, 1];
        expected.append(&mut data.clone());

        let encoded = super::encode_tobytcp(data);

        assert_eq!(expected, encoded);
    }
}

#[cfg(test)]
mod tobytcp2_tests {
    use super::*;
    #[test]
    fn mega_test() {
        assert_eq!((3, 1), decode_tobytcp2(&vec![0b0000_0011][..]));
        assert_eq!((131, 2), decode_tobytcp2(&vec![0b1000_0001, 0b0000_0011][..]));
        assert_eq!((66819, 3), decode_tobytcp2(&vec![0b1000_0100, 0b1000_1010, 0b0000_0011][..]));

        assert_eq!(vec![0b1000_0100, 0b1000_1010, 0b0000_0011], tobytcp2_prefix(66819));
        assert_eq!(vec![0b1000_0001, 0b0000_0011], tobytcp2_prefix(131));
        assert_eq!(vec![0b0000_0011], tobytcp2_prefix(3));

        let length = 338;
        let encoded_len = tobytcp2_prefix(length);

        let (decoded_length, length_length) = decode_tobytcp2(&encoded_len[..]);
        assert_eq!(length, decoded_length);
        assert_eq!(2, length_length);
    }

    #[test]
    fn loop_u7_max() {
        // for all things that can be represented in 7 bits
        for length in 0..=127 {
            let encoded_len = tobytcp2_prefix(length);
            let (decoded_length, length_length) = decode_tobytcp2(&encoded_len[..]);
            assert_eq!(length, decoded_length);
            assert_eq!(1, length_length);
        }
    }

    #[test]
    fn loop_u14_max() {
        for length in 128..=16383 {
            let encoded_len = tobytcp2_prefix(length);
            let (decoded_length, length_length) = decode_tobytcp2(&encoded_len[..]);
            assert_eq!(length, decoded_length);
            assert_eq!(2, length_length);
        }
    }
}
