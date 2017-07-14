
use std::net::TcpStream;
use std::io::Write;
use std::convert::TryFrom;

pub fn send(data: Vec<u8>, mut stream: &TcpStream) {
    let data_len_64 = u64::try_from(data.len()).unwrap();
    data_len_64.to_le();

    // Protocol says we need 8 bytes to describe the length,
    // first send the pad bytes then get to the real bytes
    stream.write(&bytes_from(data_len_64)).unwrap();

    // Write the data! Yay!
    stream.write(data.as_slice()).unwrap();
}

fn bytes_from(mut num: u64) -> [u8; 8] {
    let mut ret = [0u8; 8];

    for (i, _) in (0..7).enumerate() {
        ret[7 - i] = u8::try_from(num & 0b1111_1111_u64).unwrap();
        num = num >> 8;
    }
    ret
}
