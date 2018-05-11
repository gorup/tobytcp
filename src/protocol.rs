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
}
