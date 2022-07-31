

pub(crate) trait SizedProperty {
    fn property_len(&self) -> usize;
}

pub(crate) trait PropertyID {
    const PROPERTY_ID: u8;
}

pub(crate) trait SizedPacket {
    fn packet_len(&self) -> usize;
}

pub(crate) trait PacketID {
    const PACKET_ID: u8;
}

pub(crate) trait TryFromBytes
where
    Self: Sized,
{
    fn try_from_bytes(bytes: &[u8]) -> Option<Self>;
}

pub(crate) trait ToByteBuffer {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8];
}

pub(crate) trait TryToByteBuffer {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]>;
}

pub(crate) trait ToBytes
where
    Self: SizedProperty + ToByteBuffer,
{
    fn try_to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(self.property_len());
        self.to_byte_buffer(&mut buf);
        buf
    }
}

pub(crate) trait TryFromIterator<T>
where
    Self: Sized,
{
    fn try_from_iter<Iter>(iter: Iter) -> Option<Self>
    where
        Iter: Iterator<Item = T> + Clone;
}

pub(crate) struct ByteReader<'a> {
    buf: &'a [u8],
    offset: usize,
}

impl<'a> ByteReader<'a> {
    fn advance_by(&mut self, n: usize) {
        debug_assert!(self.offset + n <= self.buf.len());
        self.offset += n;
    }

    pub(crate) fn from(buf: &'a [u8]) -> Self {
        Self { buf, offset: 0 }
    }

    pub(crate) fn remaining(&self) -> usize {
        self.buf.len() - self.offset
    }

    pub(crate) fn offset(&self) -> usize {
        self.offset
    }

    pub(crate) fn try_read<T>(&mut self) -> Option<T>
    where
        T: Sized + TryFromBytes + SizedProperty,
    {
        let buf = &self.buf[self.offset..];
        let result = T::try_from_bytes(buf)?;
        self.advance_by(result.property_len());
        Some(result)
    }

    pub(crate) fn get_buf(&self) -> &[u8] {
        &self.buf[self.offset..]
    }
}

pub(crate) struct ByteWriter<'a> {
    buf: &'a mut [u8],
    offset: usize,
}

impl<'a> ByteWriter<'a> {
    fn advance_by(&mut self, n: usize) {
        debug_assert!(self.offset + n <= self.buf.len());
        self.offset += n;
    }

    pub(crate) fn from(buf: &'a mut [u8]) -> Self {
        Self { buf, offset: 0 }
    }

    pub(crate) fn remaining(&self) -> usize {
        self.buf.len() - self.offset
    }

    pub(crate) fn offset(&self) -> usize {
        self.offset
    }

    pub(crate) fn write<T>(&mut self, val: &T)
    where
        T: ToByteBuffer,
    {
        let buf = &mut self.buf[self.offset..];
        let written_bytes = val.to_byte_buffer(buf).len();
        self.advance_by(written_bytes);
    }

    pub(crate) fn try_write<T>(&mut self, val: &T) -> Option<()>
    where
        T: TryToByteBuffer,
    {
        let buf = &mut self.buf[self.offset..];
        let written_bytes = val.try_to_byte_buffer(buf)?.len();
        self.advance_by(written_bytes);
        Some(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod byte_writer {
        use super::*;

        #[test]
        fn write() {
            const INPUT: u32 = 0x12345678;

            let mut buf = [0u8; 32];
            let mut writer = ByteWriter::from(&mut buf);
            writer.write(&INPUT);

            assert_eq!(writer.offset(), std::mem::size_of::<u32>());
            assert_eq!(writer.remaining(), buf.len() - std::mem::size_of::<u32>());
            assert_eq!(&buf[0..std::mem::size_of::<u32>()], INPUT.to_be_bytes());
        }

        #[test]
        #[should_panic]
        fn write_out_of_bounds() {
            const INPUT: u32 = 0x12345678;

            let mut buf = [0u8; 0];
            let mut writer = ByteWriter::from(&mut buf);
            writer.write(&INPUT);
        }

        #[test]
        fn try_write() {
            const INPUT: u32 = 0x12345678;

            let mut buf = [0u8; 32];
            let mut writer = ByteWriter::from(&mut buf);
            let result = writer.try_write(&INPUT);

            assert!(result.is_some());
            assert_eq!(writer.offset(), std::mem::size_of::<u32>());
            assert_eq!(writer.remaining(), buf.len() - std::mem::size_of::<u32>());
            assert_eq!(&buf[0..std::mem::size_of::<u32>()], INPUT.to_be_bytes());
        }

        #[test]
        fn try_write_out_of_bounds() {
            const INPUT: u32 = 0x12345678;

            let mut buf = [0u8; 0];
            let mut writer = ByteWriter::from(&mut buf);
            let result = writer.try_write(&INPUT);

            assert!(result.is_none());
        }
    }

    mod byte_reader {
        use super::*;

        #[test]
        fn try_read() {
            const INPUT: [u8; 1] = [45u8];

            let mut reader = ByteReader::from(&INPUT);
            let result = reader.try_read::<u8>();

            assert!(result.is_some());
            assert_eq!(result.unwrap(), 45);
            assert_eq!(reader.offset(), 1);
            assert_eq!(reader.remaining(), 0);
        }

        #[test]
        fn try_read_out_of_bounds() {
            const INPUT: [u8; 0] = [];

            let mut reader = ByteReader::from(&INPUT);
            let result = reader.try_read::<u32>();

            assert!(result.is_none());
        }
    }
}
