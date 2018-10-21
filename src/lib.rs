pub mod protocol;

extern crate either;

use std::collections::VecDeque;
use either::{Left, Right};

/// Databuf lets you deposit data, and extract data of different lengths,
struct DataBuf {
    bufs: VecDeque<Vec<u8>>,
    total_size: u64,
}

impl Iterator for DataBuf {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

impl DataBuf {
    pub fn new() -> Self {
        DataBuf {
            bufs: VecDeque::new(),
            total_size: 0,
        }
    }

    pub fn add(&mut self, data: Vec<u8>) {
        self.total_size += data.len() as u64;
        self.bufs.push_back(data);
    }

    pub fn read(&self, index: u64) -> Option<&Vec<u8>> {
        self.bufs.get(index as usize)
    }

    // Remove length bytes from the buffer. None is returned if there are length bytes in the buf
    pub fn remove(&mut self, length: u64) -> Option<Vec<u8>> {
        if length > self.total_size || self.total_size == 0 {
            return None
        }
        // we have enough bytes

        let mut remaining_length = length;
        let mut ret = Vec::with_capacity(length as usize);
        loop {
            let mut buf = self.bufs.pop_front().unwrap();
            if buf.len() as u64 > remaining_length {
                let suffix = buf.split_off(remaining_length as usize);
                self.total_size -= buf.len() as u64;
                self.bufs.push_front(suffix);

                if !ret.is_empty() {
                    ret.append(&mut buf);
                    buf = ret;
                }
                return Some(buf);
            } else if buf.len() as u64 == remaining_length {
                self.total_size -= remaining_length;
                if !ret.is_empty() {
                    ret.append(&mut buf);
                    buf = ret;
                }
                return Some(buf);
            } else {
                remaining_length -= buf.len() as u64;
                self.total_size -= buf.len() as u64;
                ret.append(&mut buf);
            }
        }
    }
}

#[cfg(test)]
mod databuf_tests {
    use super::*;

    #[test]
    pub fn add_remove_exact() {
        let mut databuf = DataBuf::new();

        databuf.add(vec![1, 2, 3]);
        let ret = databuf.remove(3);
        assert!(ret.is_some());
        assert_eq!(vec![1, 2, 3], ret.unwrap());
    }

    #[test]
    pub fn add_remove_more() {
        let mut databuf = DataBuf::new();

        databuf.add(vec![1, 2, 3]);
        let ret = databuf.remove(31);
        assert!(ret.is_none());
    }

    #[test]
    pub fn add_add_remove_exact() {
        let mut databuf = DataBuf::new();

        databuf.add(vec![1, 2, 3]);
        databuf.add(vec![1, 2, 3]);
        let ret = databuf.remove(6);
        assert!(ret.is_some());
        assert_eq!(vec![1, 2, 3, 1, 2, 3], ret.unwrap());
    }

    #[test]
    pub fn add_add_remove_remove() {
        let mut databuf = DataBuf::new();

        databuf.add(vec![1, 2, 3]);
        databuf.add(vec![4, 5, 6]);
        let ret1 = databuf.remove(3);
        assert!(ret1.is_some());
        assert_eq!(vec![1, 2, 3], ret1.unwrap());

        let ret2 = databuf.remove(3);
        assert!(ret2.is_some());
        assert_eq!(vec![4, 5, 6], ret2.unwrap());
    }

    #[test]
    pub fn add_add_remove_add_removesome_remove_rest_check_empty() {
        let mut databuf = DataBuf::new();

        databuf.add(vec![1, 2, 3]);
        databuf.add(vec![4, 5, 6]);
        assert_eq!(vec![1, 2, 3], databuf.remove(3).unwrap());

        databuf.add(vec![7, 8, 9]);
        assert_eq!(vec![4, 5, 6, 7, 8], databuf.remove(5).unwrap());

        assert!(databuf.remove(3).is_none());

        assert_eq!(vec![9], databuf.remove(1).unwrap());

        assert!(databuf.remove(1).is_none());
    }
}

pub struct TobyReceiver {
    length: Option<(u64, u8)>,
    in_progress_length: Option<(u64, u8)>,
    ready: VecDeque<Vec<u8>>,
    databuf: DataBuf,
}

impl TobyReceiver {
    pub fn new() -> Self {
        TobyReceiver {
            length: None,
            in_progress_length: None,
            ready: VecDeque::new(),
            databuf: DataBuf::new(),
        }
    }

    pub fn bark(&mut self, data: Vec<u8>) {
        self.databuf.add(data);
        self.process();
    }

    fn process(&mut self) {
        loop {
            if self.length.is_none() {
                let mut i = 0;
                while let Some(buf) = self.databuf.read(i) {
                    i += 1;
                    match protocol::decode_tobytcp2(&buf[..], self.in_progress_length) {
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
                let (length, length_length) = self.length.unwrap();
                let all = self.databuf.remove(length + length_length as u64);
                // cut off length_length, then put a vec of length into self.ready, done!
            } else {
                break;
            }
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
