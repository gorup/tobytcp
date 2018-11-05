/// For some size, get the prefix that represents that size
pub fn tobytcp_prefix(num: usize) -> [u8; 8] {
    num.to_be_bytes()
}

/// For a buffer, grab the length (None if less than 8 bytes..)
pub fn tobytcp_length(buf: &[u8]) -> Option<u64> {
    if buf.len() < 7 {
        return None
    }
    let mut bytes = [0; 8];
    bytes.copy_from_slice(&buf[0..8]);
    Some(u64::from_be_bytes(bytes))
}

#[cfg(test)]
mod tests {
    #[test]
    fn tobytcp_prefix_u8() {
        for i in 0..=255 {
            assert_eq!([0,0,0,0,0,0,0,i], super::tobytcp_prefix(i as usize));
        }
    }

    #[test]
    fn tobytcp_prefix_spotchecks() {
        assert_eq!([0,0,0,0,0,0,1,0], super::tobytcp_prefix(256));
        assert_eq!([0,0,0,0,0,0,1,1], super::tobytcp_prefix(257));
        assert_eq!([0,0,0,0,0,0,2,0], super::tobytcp_prefix(512));
        assert_eq!([0,0,0,0,0,0,2,89], super::tobytcp_prefix(601));
        assert_eq!([0,0,0,0,0,0,4,0], super::tobytcp_prefix(1024));
        assert_eq!([0,0,0,0,0,0,8,0], super::tobytcp_prefix(2048));
        assert_eq!([0,0,0,0,0,0,9,9], super::tobytcp_prefix(2313));
    }

    #[test]
    fn tobytcp_length_u8() {
        for i in 0..=255 {
            assert_eq!(i as u64, super::tobytcp_length(&[0,0,0,0,0,0,0,i]).unwrap());
        }
    }

    #[test]
    fn tobytcp_length_spotchecks() {
        assert_eq!(257, super::tobytcp_length(&[0,0,0,0,0,0,1,1]).unwrap());
        assert_eq!(601, super::tobytcp_length(&[0,0,0,0,0,0,2,89]).unwrap());
        assert_eq!(1024, super::tobytcp_length(&[0,0,0,0,0,0,4,0]).unwrap());
        assert_eq!(14131, super::tobytcp_length(&[0,0,0,0,0,0,55,51]).unwrap());
    }

    #[test]
    fn tobytcp_equal_prefix_of_length_u8() {
        for i in 0..=255 {
            let expected = [0,0,0,0,0,0,0,i];
            assert_eq!(expected, super::tobytcp_prefix(super::tobytcp_length(&expected).unwrap() as usize));
        }
    }

    #[test]
    fn tobytcp_equal_length_of_prefix() {
        for i in 0..=1535 {
            assert_eq!(i as u64, super::tobytcp_length(&super::tobytcp_prefix(i)).unwrap());
        }
    }

    #[test]
    fn tobytcp_equal_prefix_of_length_spotchecks() {
        {
            let expected = [0,0,0,0,0,0,1,1];
            assert_eq!(expected, super::tobytcp_prefix(super::tobytcp_length(&expected).unwrap() as usize));
        }
        {
            let expected = [0,0,0,0,0,0,1,0];
            assert_eq!(expected, super::tobytcp_prefix(super::tobytcp_length(&expected).unwrap() as usize));
        }
        {
            let expected = [0,40,0,1,0,31,0,131];
            assert_eq!(expected, super::tobytcp_prefix(super::tobytcp_length(&expected).unwrap() as usize));
        }
        {
            let expected = [0,0,0,0,0,0,55,51];
            assert_eq!(expected, super::tobytcp_prefix(super::tobytcp_length(&expected).unwrap() as usize));
        }
    }
}
