use std::convert::TryFrom;

pub struct Processor {
    master_buffer: Vec<u8>,
    callback: fn(Vec<u8>),
    curr_chunk_size: Option<u64>,
}

impl Processor {
    /// Creates a new processor, callback function will
    /// be executed once a message has been received in
    /// its entirety.
    pub fn new(callback: fn(Vec<u8>)) -> Processor {
        Processor {
            master_buffer: Vec::new(),
            callback: callback,
            curr_chunk_size: None,
        }
    }

    /// Submit data for processing. Your callback will be called when a full
    /// message has been received. The execution of your callback IS BLOCKING,
    /// so if you want to process more work in parallel, have your callback
    /// function return quickly and execute asynchronously so this method can
    /// process more data
    pub fn submit(&mut self, mut buffer: Vec<u8>) {
        self.master_buffer.append(&mut buffer);
        self.set_curr_chunk_size(); // tries to set curr chunk size

        // while we know how many bytes to look for, and we have at least enough
        // bytes to process a message, process messages
        while self.curr_chunk_size.is_some()
            && self.master_buffer.len() >= to_usize(self.curr_chunk_size.unwrap()) {
                (self.callback)(self.drain_message());
                self.curr_chunk_size = None; // BAD!! We need to reset it better
                self.set_curr_chunk_size();
        }
    }

    fn set_curr_chunk_size(&mut self) {
        // If 8 bytes in buffer, and length not set, set it
        if self.master_buffer.len() >= 8 && self.curr_chunk_size.is_none() {
            self.curr_chunk_size = Some(bytes_to(&self.master_buffer[0..8]));
            self.master_buffer.drain(0..8); // get rid of the length bytes now that we know the length
        }
    }

    // might copy ugh
    fn drain_message(&mut self) -> Vec<u8> {
        self.master_buffer.drain(0..to_usize(self.curr_chunk_size.unwrap())).collect()
    }
}

/// Goes from a slice of bytes to a u64.
fn bytes_to(bytes: &[u8]) -> u64 {
    let mut ret = 0u64;
    let mut i = 0; // hacky
    for byte in bytes {
        ret = ret | u64::try_from(*byte).unwrap();
        if i < 7 {
            ret = ret << 8;
        }
        i = i + 1;
    }
    ret
}

/// Goes from a u64 to usize
/// TODO: This won't work for 32 bit machines, or at least it
/// wont if the value is greater than u32::MAX
fn to_usize(num: u64) -> usize {
    usize::try_from(num).unwrap()
}
