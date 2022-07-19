pub trait SizedProperty {
    fn property_len(&self) -> usize;
}

pub trait TryFromBytes
where
    Self: Sized,
{
    fn try_from_bytes(bytes: &[u8]) -> Option<Self>;
}

pub trait ToByteBuffer {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8];
}

pub trait TryToByteBuffer {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]>;
}

pub trait ToBytes
where
    Self: SizedProperty + ToByteBuffer,
{
    fn try_to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.property_len());
        self.to_byte_buffer(&mut buf);
        buf
    }
}

pub trait TryFromIterator<T>
where
    Self: Sized,
{
    fn try_from_iter<Iter>(iter: Iter) -> Option<Self>
    where
        Iter: Iterator<Item = T> + Clone;
}

pub struct ByteWriter<'a> {
    buf: &'a mut [u8],
    offset: usize,
}

impl<'a> ByteWriter<'a> {
    pub fn from(buf: &'a mut [u8]) -> Self {
        Self { buf, offset: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.offset - self.buf.len()
    }

    pub fn advance_by(&mut self, n: usize) {
        debug_assert!(self.offset + n <= self.buf.len());
        self.offset += n;
    }

    pub fn try_advance_by(&mut self, n: usize) -> Option<()> {
        if self.offset + n <= self.buf.len() {
            return None;
        }

        self.advance_by(n);
        Some(())
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn write<T>(&mut self, val: &T)
    where
        T: ToByteBuffer,
    {
        let buf = &mut self.buf[self.offset..];
        let written_bytes = val.to_byte_buffer(buf).len();
        self.advance_by(written_bytes);
    }

    pub fn try_write<T>(&mut self, val: &T) -> Option<()>
    where
        T: TryToByteBuffer,
    {
        let buf = &mut self.buf[self.offset..];
        let written_bytes = val.try_to_byte_buffer(buf)?.len();
        self.try_advance_by(written_bytes)
    }
}
