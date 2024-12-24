use std::cell::RefCell;
use std::collections::VecDeque;
use std::io::{Read, Result};
use std::rc::Rc;

struct IOStub {
    reads: Rc<RefCell<VecDeque<Result<Vec<u8>>>>>,
}

impl IOStub {
    pub fn new() -> Self {
        Self {
            reads: Rc::new(RefCell::new(VecDeque::new())),
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            reads: self.reads.clone(),
        }
    }

    pub fn push_read_error(&mut self, e: std::io::Error) {
        self.reads.borrow_mut().push_back(Err(e));
    }

    pub fn push_read(&mut self, bytes: &[u8]) {
        self.reads.borrow_mut().push_back(Ok(Vec::from(bytes)));
    }
}

impl Read for IOStub {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // try to read from reads queue
        let bytes = self.reads.borrow_mut().pop_front();
        if bytes.is_none() {
            // pop_front failure should result in an read of size zero
            // meaning there is nothing left to read
            return Ok(0);
        }
        let bytes = bytes.unwrap();

        // return an std::io::Error if any
        let bytes = bytes?;

        // split vector
        let wsz = usize::min(bytes.len(), buf.len());
        let (copy, rem) = bytes.split_at(wsz);
        // copy data to output buffer
        buf[..wsz].copy_from_slice(copy);

        // if any push back any remaining bytes
        if rem.len() > 0 {
            self.reads.borrow_mut().push_front(Ok(Vec::from(rem)));
        }

        // return written length
        Ok(wsz)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error, ErrorKind};

    struct ConsumeReader<R> {
        reader: R,
    }

    impl<R: Read> ConsumeReader<R> {
        pub fn new(reader: R) -> Self {
            Self { reader }
        }

        pub fn read_one(&mut self) -> std::io::Result<Vec<u8>> {
            let mut vec = vec![0u8; 256];
            let rsz = self.reader.read(&mut vec)?;
            vec.resize(rsz, 0xffu8);
            Ok(vec)
        }

        pub fn read_all(&mut self) -> std::io::Result<Vec<u8>> {
            let mut vec = vec![];
            self.reader.read_to_end(&mut vec)?;
            Ok(vec)
        }
    }

    #[test]
    fn consecutive_reads() {
        let mut stub = IOStub::new();
        let mut cr = ConsumeReader::new(stub.clone());
        stub.push_read(b"otters");
        stub.push_read(b"are");
        stub.push_read(b"amazing");
        let rv = cr.read_one();
        assert!(rv.is_ok());
        assert_eq!(rv.unwrap(), Vec::from(b"otters"));
        let rv = cr.read_one();
        assert!(rv.is_ok());
        assert_eq!(rv.unwrap(), Vec::from(b"are"));
        let rv = cr.read_one();
        assert!(rv.is_ok());
        assert_eq!(rv.unwrap(), Vec::from(b"amazing"));
    }

    #[test]
    fn read_all() {
        let mut stub = IOStub::new();
        let mut cr = ConsumeReader::new(stub.clone());
        stub.push_read(b"otters");
        stub.push_read(b"are");
        stub.push_read(b"amazing");
        let rv = cr.read_all();
        assert!(rv.is_ok());
        assert_eq!(rv.unwrap(), Vec::from(b"ottersareamazing"));
    }

    #[test]
    fn read_all_interleaved() {
        let mut stub = IOStub::new();
        let mut cr = ConsumeReader::new(stub.clone());
        stub.push_read(b"otters");
        stub.push_read(b"are");
        stub.push_read(b"amazing");
        let rv = cr.read_all();
        assert!(rv.is_ok());
        assert_eq!(rv.unwrap(), Vec::from(b"ottersareamazing"));
        let rv = cr.read_all();
        assert!(rv.is_ok());
        assert_eq!(rv.unwrap(), Vec::from(b""));
        stub.push_read(b"from");
        stub.push_read(b"otter");
        stub.push_read(b"space");
        let rv = cr.read_all();
        assert!(rv.is_ok());
        assert_eq!(rv.unwrap(), Vec::from(b"fromotterspace"));
    }

    #[test]
    fn read_error() {
        let mut stub = IOStub::new();
        let mut cr = ConsumeReader::new(stub.clone());
        stub.push_read(b"otters");
        stub.push_read(b"are");
        stub.push_read_error(Error::new(ErrorKind::TimedOut, "xxx"));
        stub.push_read(b"amazing");
        let rv = cr.read_one();
        assert!(rv.is_ok());
        assert_eq!(rv.unwrap(), Vec::from(b"otters"));
        let rv = cr.read_one();
        assert!(rv.is_ok());
        assert_eq!(rv.unwrap(), Vec::from(b"are"));
        let rv = cr.read_one();
        assert!(rv.is_err());
        let e = rv.unwrap_err();
        assert_eq!(e.kind(), ErrorKind::TimedOut);
        let rv = cr.read_one();
        assert!(rv.is_ok());
        assert_eq!(rv.unwrap(), Vec::from(b"amazing"));
    }
}
