use std::marker::PhantomData;

use bytes::{Buf, Bytes, BytesMut};

pub(crate) trait ByteLen {
    fn byte_len(&self) -> usize;
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

pub(crate) trait Encode {
    fn encode(&self, buf: &mut BytesMut);
}

pub(crate) trait TryEncode
where
    Self: Sized,
{
    type Error;

    fn try_encode(&self, buf: &mut BytesMut) -> Result<(), Self::Error>;
}

pub(crate) trait Decode {
    fn decode(buf: Bytes) -> Self;
}

pub(crate) trait TryDecode
where
    Self: Sized,
{
    type Error;

    fn try_decode(buf: Bytes) -> Result<Self, Self::Error>;
}

pub(crate) struct DecodeIter<T> {
    decoder: Decoder,
    _phantom: PhantomData<T>,
}

impl<T> Iterator for DecodeIter<T>
where
    T: ByteLen + TryDecode,
{
    type Item = Result<T, T::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.decoder.remaining() == 0 {
            None
        } else {
            Some(self.decoder.try_decode::<T>())
        }
    }
}

#[derive(Clone)]
pub(crate) struct Decoder {
    buf: Bytes,
}

impl From<Bytes> for Decoder {
    fn from(buf: Bytes) -> Self {
        Self { buf }
    }
}

impl Decoder {
    pub(crate) fn advance_by(&mut self, n: usize) {
        self.buf.advance(n);
    }

    pub(crate) fn remaining(&self) -> usize {
        self.buf.len()
    }

    pub(crate) fn try_decode<T>(&mut self) -> Result<T, T::Error>
    where
        T: Sized + TryDecode + ByteLen,
    {
        let result = T::try_decode(self.buf.clone())?;
        self.advance_by(result.byte_len());
        Ok(result)
    }

    pub(crate) fn get_buf(&self) -> Bytes {
        self.buf.clone()
    }

    pub(crate) fn iter<T>(self) -> DecodeIter<T>
    where
        T: TryDecode,
    {
        DecodeIter {
            decoder: self,
            _phantom: PhantomData,
        }
    }
}

pub(crate) struct Encoder<'a> {
    buf: &'a mut BytesMut,
}

impl<'a> From<&'a mut BytesMut> for Encoder<'a> {
    fn from(buf: &'a mut BytesMut) -> Self {
        Self { buf }
    }
}

impl<'a> Encoder<'a> {
    pub(crate) fn encode<T>(&mut self, val: T)
    where
        T: Encode,
    {
        val.encode(&mut self.buf)
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;

//     mod encoder {
//         use super::*;

//         #[test]
//         fn write_u32() {
//             const INPUT: u32 = 0x12345678;

//             let mut writer = Encoder::from(BytesMut::new());
//             writer.try_encode::<u32>(INPUT);

//             assert_eq!(
//                 &writer.get_buf()[0..std::mem::size_of::<u32>()],
//                 INPUT.to_be_bytes()
//             );
//         }
//     }

//     mod decoder {
//         use super::*;

//         #[test]
//         fn try_read() {
//             const INPUT: [u8; 1] = [45u8];
//             let buf = Bytes::from_static(&INPUT);

//             let mut reader = Decoder::from(buf);
//             let result = reader.try_decode::<u8>();

//             assert!(result.is_ok());
//             assert_eq!(result.unwrap(), 45);
//             assert_eq!(reader.remaining(), 0);
//         }

//         #[test]
//         fn try_read_out_of_bounds() {
//             const INPUT: [u8; 0] = [];
//             let buf = Bytes::from_static(&INPUT);

//             let mut reader = Decoder::from(buf);
//             let result = reader.try_decode::<u32>();

//             assert!(result.is_err());
//         }
//     }
// }
