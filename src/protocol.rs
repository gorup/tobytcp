use std::convert::TryFrom;

/// Call this with your data, and the returned buffer will be a properly
/// encoded `TobyTcp` message that can be sent!
pub fn encode_tobytcp(mut message: Vec<u8>) -> Vec<u8> {
    let data_len_64 = u64::try_from(message.len()).unwrap();
    data_len_64.to_le();

    let mut encoded = bytes_from(data_len_64).to_vec();
    encoded.append(&mut message);
    encoded
}

/// Goes from a single u64 to 8xu8
fn bytes_from(mut num: u64) -> [u8; 8] {
    let mut ret = [0u8; 8];

    for (i, _) in (0..7).enumerate() {
        ret[7 - i] = u8::try_from(num & 0b1111_1111_u64).unwrap();
        num = num >> 8;
    }
    ret
}
