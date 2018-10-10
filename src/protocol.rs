//! This module has a helper for encoding data to TobyTcp

/// Call this with your data, and the returned buffer will be a properly
/// encoded `TobyTcp` message that can be sent!
pub fn encode_tobytcp(mut message: Vec<u8>) -> Vec<u8> {
    let data_len_64 = message.len() as u64;
    data_len_64.to_le();

    let mut encoded = bytes_from(data_len_64).to_vec();
    encoded.append(&mut message);
    encoded
}

/// Goes from a single u64 to 8xu8
fn bytes_from(mut num: u64) -> [u8; 8] {
    let mut ret = [0u8; 8];

    for (i, _) in (0..7).enumerate() {
        ret[7 - i] = (num & 0b1111_1111_u64) as u8;
        num = num >> 8;
    }
    ret
}

    // the left-most bit defines if the next bit is needed
    // if you read the right 7 bits, thats the length, the left
    // most bit is never part of the size
// grows like toby
pub fn encode_tobytcp2(mut message: Vec<u8>) -> Vec<u8> {
    let mut prefix = tobytcp2_prefix(message.len() as u64);

    prefix.append(&mut message);
    prefix
}

pub fn tobytcp2_prefix(mut num_bytes: u64) -> Vec<u8> {
    let mut bytes_backwards: Vec<u8> = Vec::new();

    // while the number is greater than u7 max..
    while num_bytes > (u8::max_value() >> 1).into() {
        bytes_backwards.push((num_bytes | 0b1000_0000_u64) as u8);
        num_bytes = num_bytes >> 7;
    }
    bytes_backwards.push((num_bytes | 0b0000_0000_u64) as u8);
    bytes_backwards.reverse();
    bytes_backwards
}

#[cfg(test)]
mod tests {
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

    #[test]
    fn encoded2() {
        let message = vec![100, 13, 69, 17];
        let encoded = super::encode_tobytcp2(message);
        // We had 4 bytes of data
        assert_eq!(vec![4, 100, 13, 69, 17], encoded);
    }

    #[test]
    fn encoded2_two_bytes() {
        let encoded_pre = super::tobytcp2_prefix(127);
        assert_eq!(vec![0b0111_1111], encoded_pre);

        let encoded_barely = super::tobytcp2_prefix(128);
        assert_eq!(vec![0b0000_0001, 0b1000_0000], encoded_barely);

        let encoded_max = super::tobytcp2_prefix(255);
        assert_eq!(vec![0b0000_0001, 0b1111_1111], encoded_max);

        let encoded = super::tobytcp2_prefix(256);
        assert_eq!(vec![0b0000_0010, 0b1000_0000], encoded);

        let encoded2 = super::tobytcp2_prefix(257);
        assert_eq!(vec![0b0000_0010, 0b1000_0001], encoded2);

        let encoded3 = super::tobytcp2_prefix(258);
        assert_eq!(vec![0b0000_0010, 0b1000_0010], encoded3);

        let encoded4 = super::tobytcp2_prefix(259);
        assert_eq!(vec![0b0000_0010, 0b1000_0011], encoded4);

        let encoded5 = super::tobytcp2_prefix(683);
        assert_eq!(vec![0b0000_0101, 0b1010_1011], encoded5);
    }

    #[test]
    fn encoded2_4_bytes() {
        //72143031
        //   0100 0100 1100 1101 0000 1011 0111_2
        //   010 0010 011 0011 010 0001 011 0111_2
        let encoded = super::tobytcp2_prefix(72143031);
        assert_eq!(vec![0b0010_0010, 0b1011_0011, 0b1010_0001, 0b1011_0111], encoded);
    }
}
