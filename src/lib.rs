pub mod protocol;

use std::collections::VecDeque;

pub struct TobyReceiver {
    raw_buff: Vec<u8>,
    curr_size: Option<u64>,
    ready: VecDeque<Vec<u8>>,
}

impl TobyReceiver {
    fn new() -> Self {
        TobyReceiver {
            curr_size: None,
            raw_buff: Vec::new(),
            ready: VecDeque::new(),
        }
    }

    // consumes tcp stream, sends finished messages to Sender's corresponding receiver
    fn bark(&mut self, mut data: Vec<u8>) {
        self.raw_buff.append(&mut data);

        self.curr_size = compute_curr_size(self.curr_size, &mut self.raw_buff);
        while self.curr_size.is_some() && self.raw_buff.len() >= self.curr_size.unwrap() as usize {
            // get the data out!
            let parsed_message = self.raw_buff.drain(0..self.curr_size.unwrap() as usize).collect();

            // reset the size, ugly!
            self.curr_size = compute_curr_size(None, &mut self.raw_buff);

            self.ready.push_back(parsed_message);
        }
    }
}

impl Iterator for TobyReceiver {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        self.ready.pop_front()
    }
}

fn compute_curr_size(curr_size: Option<u64>, buf: &mut Vec<u8>) -> Option<u64> {
    if curr_size.is_none() {
        if buf.len() >= 8 {
            let size = Some(bytes_to(&buf[0..8]));
            buf.drain(0..8);
            return size;
        }
        None
    } else {
        curr_size
    }
}

/// Goes from a slice of bytes to a u64.
fn bytes_to(bytes: &[u8]) -> u64 {
    let mut ret = 0u64;
    let mut i = 0; // hacky
    for byte in bytes {
        ret = ret | *byte as u64;
        if i < 7 {
            ret = ret << 8;
        }
        i = i + 1;
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::protocol;
    use super::TobyReceiver;

    #[test]
    pub fn send_one_receive_one() {
        let data = vec![35; 31];
        let verify = data.clone();

        let encoded = protocol::encode_tobytcp(data);

        let mut tobyr = TobyReceiver::new();
        tobyr.bark(encoded);
        assert_eq!(verify, tobyr.next().unwrap());
        assert_eq!(None, tobyr.next());
    }

    #[test]
    pub fn send_partial_only_receive_once_all_sent() {
        let mut data = vec![101; 88];
        let verify = data.clone();

        let mut encoded = protocol::encode_tobytcp(data);

        let first_byte = encoded.drain(0..1).collect();

        let mut tobyr = TobyReceiver::new();

        tobyr.bark(first_byte);
        assert_eq!(None, tobyr.next());

        let len = encoded.len() / 2;
        let more_bytes = encoded.drain(0..len).collect();
        tobyr.bark(more_bytes);
        assert_eq!(None, tobyr.next());

        tobyr.bark(encoded);
        assert_eq!(verify, tobyr.next().unwrap());
    }

    #[test]
    pub fn iterating_over_a_handful() {
        let mut tobyr = TobyReceiver::new();
        for i in 0..69 {
            let data = vec![i];
            let verify = data.clone();

            let encoded = protocol::encode_tobytcp(data);
            tobyr.bark(encoded);
        }

        for (i, data) in tobyr.enumerate() {
            assert_eq!(vec![i as u8], data)
        }
    }
}
