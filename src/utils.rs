use ::bytes;

use ::std::fmt::{Write, Display};
use ::std::borrow::Borrow;
use ::std::io;

pub fn comma_delimited<T: Display + ?Sized, B: Borrow<T>, I: IntoIterator<Item=B>>(iter: I) -> String {
    let mut iter = iter.into_iter();
    let mut ret = String::new();
    if let Some(b) = iter.next() {
        write!(ret, "{}", b.borrow()).unwrap();
        for b in iter {
            write!(ret, ",{}", b.borrow()).unwrap();
        }
    }
    ret
}

//const DEFAULT_CAPACITY: usize = 4096;
const SMOL_CAPCITY: usize = 64;

pub(crate) struct BytesWriter {
    buf: bytes::BytesMut,
}

impl BytesWriter {
    //#[inline]
    //pub fn new() -> Self {
    //    Self::with_capacity(DEFAULT_CAPACITY)
    //}

    #[inline]
    pub fn with_smol_capacity() -> Self {
        Self::with_capacity(SMOL_CAPCITY)
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: bytes::BytesMut::with_capacity(capacity)
        }
    }

    #[inline]
    pub fn into_inner(self) -> bytes::BytesMut {
        self.buf
    }

    //#[inline]
    //pub fn freeze(&mut self) -> bytes::Bytes {
    //    mem::replace(&mut self.buf, bytes::BytesMut::new()).freeze()
    //}

    //#[inline]
    //pub fn len(&self) -> usize {
    //    self.buf.len()
    //}

    //#[inline]
    //pub fn split_off(&mut self, at: usize) -> Self {
    //    Self {
    //        buf: self.buf.split_off(at)
    //    }
    //}

    //#[inline]
    //pub fn reserve(&mut self, add: usize) {
    //    self.buf.reserve(add);
    //}
}

impl io::Write for BytesWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buf.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.buf.extend_from_slice(buf);
        Ok(())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
