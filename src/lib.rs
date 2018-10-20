pub mod protocol;

extern crate either;

use std::collections::VecDeque;
use either::{Either, Left, Right};

pub struct TobyReceiver {
    length: Option<(u64, u8)>,
    in_progress_length: Option<(u64, u8)>,
    ready: VecDeque<Vec<u8>>,
    unready: VecDeque<Vec<u8>>,
}

impl TobyReceiver {
    pub fn new() -> Self {
        TobyReceiver {
            length: None, in_progress_length: None,
            ready: VecDeque::new(), unready: VecDeque::new(),
        }
    }

    pub fn bark(&mut self, mut data: Vec<u8>) {
        self.unready.push_back(data);
        self.process();
    }

    fn process(&mut self) {
        if self.length.is_none() {
            let mut prev_right = None;

            // Provide decode with our bufs until we get a length
            for buf in self.unready.iter() {
                match protocol::decode_tobytcp2(&buf[..], prev_right) {
                    Left(finished) => {
                        self.length = Some(finished);
                        self.in_progress_length = None;
                        break;
                    }
                    Right(unfinished) => self.in_progress_length = Some(unfinished),
                }
            }
        }

        if self.length.is_some() {
        }
    }
}

impl Iterator for TobyReceiver {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        self.ready.pop_front()
    }
}

#[cfg(test)]
mod tests {
    /*
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
*/
}
