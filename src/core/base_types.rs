use crate::core::{
    error::{
        ConversionError, InsufficientBufferSize, InvalidEncoding, InvalidValue,
        ValueExceedesMaximum, ValueIsZero,
    },
    utils::{ByteLen, Encode, TryDecode},
};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use core::{
    convert::From,
    iter::Iterator,
    mem,
    ops::{Add, Div, Mul, Sub},
};

#[allow(clippy::enum_variant_names)]
#[derive(Copy, Clone, PartialEq, Debug, Eq, PartialOrd)]
enum VarSizeIntState {
    SingleByte(u8),
    TwoByte(u16),
    ThreeByte(u32),
    FourByte(u32),
}

#[derive(Copy, Clone, PartialEq, Debug, Eq)]
pub(crate) struct VarSizeInt(VarSizeIntState);

impl VarSizeInt {
    pub(crate) const MAX: usize = 0x0fffffff;

    pub(crate) fn len(&self) -> usize {
        match self.0 {
            VarSizeIntState::SingleByte(_) => 1,
            VarSizeIntState::TwoByte(_) => 2,
            VarSizeIntState::ThreeByte(_) => 3,
            VarSizeIntState::FourByte(_) => 4,
        }
    }

    pub(crate) fn value(&self) -> u32 {
        match self.0 {
            VarSizeIntState::SingleByte(val) => val as u32,
            VarSizeIntState::TwoByte(val) => val as u32,
            VarSizeIntState::ThreeByte(val) => val as u32,
            VarSizeIntState::FourByte(val) => val as u32,
        }
    }
}

impl TryFrom<&[u8]> for VarSizeInt {
    type Error = ConversionError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut mult = 1u32;
        let mut val = 0u32;

        for (idx, &byte) in bytes.iter().enumerate() {
            val += (byte as u32 & 127) * mult;

            if mult as usize > Self::MAX {
                return Err(ValueExceedesMaximum.into());
            }

            mult *= 128;

            if byte & 128 == 0 {
                return match idx {
                    0 => Ok(Self(VarSizeIntState::SingleByte(val as u8))),
                    1 => Ok(Self(VarSizeIntState::TwoByte(val as u16))),
                    2 => Ok(Self(VarSizeIntState::ThreeByte(val as u32))),
                    3 => Ok(Self(VarSizeIntState::FourByte(val as u32))),
                    _ => Err(InvalidEncoding.into()),
                };
            }
        }

        Err(InsufficientBufferSize.into())
    }
}

impl Default for VarSizeInt {
    fn default() -> Self {
        Self(VarSizeIntState::SingleByte(0))
    }
}

impl ByteLen for VarSizeInt {
    fn byte_len(&self) -> usize {
        self.len()
    }
}

impl TryDecode for VarSizeInt {
    type Error = ConversionError;

    fn try_decode(bytes: Bytes) -> Result<Self, Self::Error> {
        Self::try_from(bytes.as_ref())
    }
}

impl Encode for VarSizeInt {
    fn encode(&self, buf: &mut BytesMut) {
        match self.0 {
            VarSizeIntState::SingleByte(val) => {
                buf.put_u8(val);
            }
            VarSizeIntState::TwoByte(mut val) => {
                let byte0 = (val % 0x80) as u8 | 0x80;
                val /= 0x80;
                let byte1 = (val % 0x80) as u8;
                debug_assert!(val / 0x80 == 0);

                buf.put(&[byte0, byte1][..]);
            }
            VarSizeIntState::ThreeByte(mut val) => {
                let byte0 = (val % 0x80) as u8 | 0x80;
                val /= 0x80;
                let byte1 = (val % 0x80) as u8 | 0x80;
                val /= 0x80;
                let byte2 = (val % 0x80) as u8;
                debug_assert!(val / 0x80 == 0);

                buf.put(&[byte0, byte1, byte2][..]);
            }

            VarSizeIntState::FourByte(mut val) => {
                let byte0 = (val % 0x80) as u8 | 0x80;
                val /= 0x80;
                let byte1 = (val % 0x80) as u8 | 0x80;
                val /= 0x80;
                let byte2 = (val % 0x80) as u8 | 0x80;
                val /= 0x80;
                let byte3 = (val % 0x80) as u8;
                debug_assert!(val / 0x80 == 0);

                buf.put(&[byte0, byte1, byte2, byte3][..]);
            }
        }
    }
}

impl PartialEq<u8> for VarSizeInt {
    fn eq(&self, other: &u8) -> bool {
        self.value() == *other as u32
    }
}

impl PartialEq<u16> for VarSizeInt {
    fn eq(&self, other: &u16) -> bool {
        self.value() == *other as u32
    }
}

impl PartialEq<u32> for VarSizeInt {
    fn eq(&self, other: &u32) -> bool {
        self.value() == *other
    }
}

impl PartialEq<usize> for VarSizeInt {
    fn eq(&self, other: &usize) -> bool {
        self.value() as usize == *other
    }
}

impl PartialEq<i8> for VarSizeInt {
    fn eq(&self, other: &i8) -> bool {
        if *other > 0 {
            self.value() == *other as u32
        } else {
            false
        }
    }
}

impl PartialEq<i16> for VarSizeInt {
    fn eq(&self, other: &i16) -> bool {
        if *other > 0 {
            self.value() == *other as u32
        } else {
            false
        }
    }
}

impl PartialEq<i32> for VarSizeInt {
    fn eq(&self, other: &i32) -> bool {
        if *other > 0 {
            self.value() == *other as u32
        } else {
            false
        }
    }
}

impl PartialEq<isize> for VarSizeInt {
    fn eq(&self, other: &isize) -> bool {
        if *other > 0 {
            self.value() as usize == *other as usize
        } else {
            false
        }
    }
}

impl PartialOrd<u8> for VarSizeInt {
    fn partial_cmp(&self, other: &u8) -> Option<std::cmp::Ordering> {
        Some(self.value().cmp(&(*other as u32)))
    }
}

impl PartialOrd<u16> for VarSizeInt {
    fn partial_cmp(&self, other: &u16) -> Option<std::cmp::Ordering> {
        Some(self.value().cmp(&(*other as u32)))
    }
}

impl PartialOrd<u32> for VarSizeInt {
    fn partial_cmp(&self, other: &u32) -> Option<std::cmp::Ordering> {
        Some(self.value().cmp(other))
    }
}

impl PartialOrd<usize> for VarSizeInt {
    fn partial_cmp(&self, other: &usize) -> Option<std::cmp::Ordering> {
        Some((self.value() as usize).cmp(other))
    }
}

impl PartialOrd<i8> for VarSizeInt {
    fn partial_cmp(&self, other: &i8) -> Option<std::cmp::Ordering> {
        if *other > 0 {
            Some((self.value()).cmp(&(*other as u32)))
        } else {
            Some(std::cmp::Ordering::Greater)
        }
    }
}

impl PartialOrd<i16> for VarSizeInt {
    fn partial_cmp(&self, other: &i16) -> Option<std::cmp::Ordering> {
        if *other > 0 {
            Some((self.value()).cmp(&(*other as u32)))
        } else {
            Some(std::cmp::Ordering::Greater)
        }
    }
}

impl PartialOrd<i32> for VarSizeInt {
    fn partial_cmp(&self, other: &i32) -> Option<std::cmp::Ordering> {
        if *other > 0 {
            Some((self.value()).cmp(&(*other as u32)))
        } else {
            Some(std::cmp::Ordering::Greater)
        }
    }
}

impl PartialOrd<isize> for VarSizeInt {
    fn partial_cmp(&self, other: &isize) -> Option<std::cmp::Ordering> {
        if *other > 0 {
            Some((self.value() as usize).cmp(&(*other as usize)))
        } else {
            Some(std::cmp::Ordering::Greater)
        }
    }
}

impl Add for VarSizeInt {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::try_from(self.value() + rhs.value()).unwrap()
    }
}

impl Sub for VarSizeInt {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::try_from(self.value() - rhs.value()).unwrap()
    }
}

impl Mul for VarSizeInt {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::try_from(self.value() * rhs.value()).unwrap()
    }
}

impl Div for VarSizeInt {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self::try_from(self.value() / rhs.value()).unwrap()
    }
}

impl From<u8> for VarSizeInt {
    fn from(val: u8) -> Self {
        if val <= 127 {
            Self(VarSizeIntState::SingleByte(val))
        } else {
            Self(VarSizeIntState::TwoByte(val as u16))
        }
    }
}

impl From<u16> for VarSizeInt {
    fn from(val: u16) -> Self {
        if val <= 127 {
            return Self(VarSizeIntState::SingleByte(val as u8));
        } else if (128..=16383).contains(&val) {
            return Self(VarSizeIntState::TwoByte(val));
        }

        Self(VarSizeIntState::ThreeByte(val as u32))
    }
}

impl TryFrom<u32> for VarSizeInt {
    type Error = ConversionError;

    fn try_from(val: u32) -> Result<Self, Self::Error> {
        if val <= 127 {
            Ok(Self(VarSizeIntState::SingleByte(val as u8)))
        } else if (128..=16383).contains(&val) {
            Ok(Self(VarSizeIntState::TwoByte(val as u16)))
        } else if (16384..=2097151).contains(&val) {
            Ok(Self(VarSizeIntState::ThreeByte(val)))
        } else if val as usize <= Self::MAX {
            Ok(Self(VarSizeIntState::FourByte(val)))
        } else {
            Err(ValueExceedesMaximum.into())
        }
    }
}

impl TryFrom<usize> for VarSizeInt {
    type Error = ConversionError;

    fn try_from(val: usize) -> Result<Self, Self::Error> {
        if val <= 127 {
            Ok(Self(VarSizeIntState::SingleByte(val as u8)))
        } else if (128..=16383).contains(&val) {
            Ok(Self(VarSizeIntState::TwoByte(val as u16)))
        } else if (16384..=2097151).contains(&val) {
            Ok(Self(VarSizeIntState::ThreeByte(val as u32)))
        } else if val <= Self::MAX {
            Ok(Self(VarSizeIntState::FourByte(val as u32)))
        } else {
            Err(ValueExceedesMaximum.into())
        }
    }
}

impl TryFrom<VarSizeInt> for u8 {
    type Error = ConversionError;

    fn try_from(val: VarSizeInt) -> Result<Self, Self::Error> {
        match val.0 {
            VarSizeIntState::SingleByte(val) => Ok(val as u8),
            VarSizeIntState::TwoByte(val) => {
                if val > 0xff {
                    Err(ValueExceedesMaximum.into())
                } else {
                    Ok(val as u8)
                }
            }
            _ => Err(ValueExceedesMaximum.into()),
        }
    }
}

impl TryFrom<VarSizeInt> for u16 {
    type Error = ConversionError;

    fn try_from(val: VarSizeInt) -> Result<Self, Self::Error> {
        match val.0 {
            VarSizeIntState::SingleByte(val) => Ok(val as u16),
            VarSizeIntState::TwoByte(val) => Ok(val as u16),
            VarSizeIntState::ThreeByte(val) => {
                if val > 0xffff {
                    Err(ValueExceedesMaximum.into())
                } else {
                    Ok(val as u16)
                }
            }
            _ => Err(ValueExceedesMaximum.into()),
        }
    }
}

impl From<VarSizeInt> for u32 {
    fn from(val: VarSizeInt) -> Self {
        match val.0 {
            VarSizeIntState::SingleByte(val) => val as u32,
            VarSizeIntState::TwoByte(val) => val as u32,
            VarSizeIntState::ThreeByte(val) => val,
            VarSizeIntState::FourByte(val) => val,
        }
    }
}

impl From<VarSizeInt> for usize {
    fn from(val: VarSizeInt) -> Self {
        match val.0 {
            VarSizeIntState::SingleByte(val) => val as usize,
            VarSizeIntState::TwoByte(val) => val as usize,
            VarSizeIntState::ThreeByte(val) => val as usize,
            VarSizeIntState::FourByte(val) => val as usize,
        }
    }
}

impl ByteLen for u8 {
    fn byte_len(&self) -> usize {
        mem::size_of::<Self>()
    }
}

impl TryDecode for u8 {
    type Error = ConversionError;

    fn try_decode(bytes: Bytes) -> Result<Self, Self::Error> {
        bytes
            .first()
            .copied()
            .ok_or_else(|| InsufficientBufferSize.into())
    }
}

impl Encode for u8 {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u8(*self);
    }
}

/// Enum representing Quality Of Service
///
#[allow(clippy::enum_variant_names)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum QoS {
    /// At most once QoS
    ///
    AtMostOnce = 0,

    /// At least once QoS
    ///
    AtLeastOnce = 1,

    /// Exactly once QoS
    ///
    ExactlyOnce = 2,
}

impl TryFrom<u8> for QoS {
    type Error = ConversionError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(QoS::AtMostOnce),
            1 => Ok(QoS::AtLeastOnce),
            2 => Ok(QoS::ExactlyOnce),
            _ => Err(InvalidValue.into()),
        }
    }
}

impl Default for QoS {
    fn default() -> Self {
        QoS::AtMostOnce
    }
}

impl ByteLen for QoS {
    fn byte_len(&self) -> usize {
        mem::size_of::<u8>()
    }
}

impl TryDecode for QoS {
    type Error = ConversionError;

    fn try_decode(bytes: Bytes) -> Result<Self, Self::Error> {
        bytes
            .first()
            .copied()
            .ok_or_else(|| InsufficientBufferSize.into())
            .and_then(Self::try_from)
    }
}

impl Encode for QoS {
    fn encode(&self, buf: &mut BytesMut) {
        (*self as u8).encode(buf)
    }
}

impl ByteLen for bool {
    fn byte_len(&self) -> usize {
        mem::size_of::<u8>()
    }
}

impl TryDecode for bool {
    type Error = ConversionError;

    fn try_decode(bytes: Bytes) -> Result<Self, Self::Error> {
        bytes
            .first()
            .ok_or_else(|| InsufficientBufferSize.into())
            .and_then(|val| match val {
                0u8 => Ok(false),
                1u8 => Ok(true),
                _ => Err(InvalidValue.into()),
            })
    }
}

impl Encode for bool {
    fn encode(&self, buf: &mut BytesMut) {
        (*self as u8).encode(buf)
    }
}

impl ByteLen for u16 {
    fn byte_len(&self) -> usize {
        mem::size_of::<Self>()
    }
}

impl TryDecode for u16 {
    type Error = ConversionError;

    fn try_decode(bytes: Bytes) -> Result<Self, Self::Error> {
        bytes
            .iter()
            .take(mem::size_of::<u16>())
            .map(|&value| value as u16)
            .reduce(|result, tmp| result << 8 | tmp)
            .ok_or_else(|| InsufficientBufferSize.into())
    }
}

impl Encode for u16 {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u16(*self);
    }
}

impl ByteLen for u32 {
    fn byte_len(&self) -> usize {
        mem::size_of::<Self>()
    }
}

impl Encode for u32 {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u32(*self);
    }
}

impl TryDecode for u32 {
    type Error = ConversionError;

    fn try_decode(bytes: Bytes) -> Result<Self, Self::Error> {
        bytes
            .iter()
            .take(mem::size_of::<u32>())
            .map(|&value| value as u32)
            .reduce(|result, tmp| result << 8 | tmp)
            .ok_or_else(|| InsufficientBufferSize.into())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct NonZero<T>(T)
where
    T: Copy;

impl<T> NonZero<T>
where
    T: Copy,
{
    pub(crate) fn get(&self) -> T {
        self.0
    }
}

impl<T> PartialEq<T> for NonZero<T>
where
    T: Copy + PartialEq,
{
    fn eq(&self, other: &T) -> bool {
        self.get() == *other
    }
}

impl TryFrom<u8> for NonZero<u8> {
    type Error = ConversionError;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        if val == 0 {
            return Err(ValueIsZero.into());
        }

        Ok(Self(val))
    }
}

impl From<NonZero<u8>> for u8 {
    fn from(val: NonZero<u8>) -> Self {
        val.get()
    }
}

impl ByteLen for NonZero<u8> {
    fn byte_len(&self) -> usize {
        self.get().byte_len()
    }
}

impl Encode for NonZero<u8> {
    fn encode(&self, buf: &mut BytesMut) {
        self.get().encode(buf)
    }
}

impl TryDecode for NonZero<u8> {
    type Error = ConversionError;

    fn try_decode(bytes: Bytes) -> Result<Self, Self::Error> {
        u8::try_decode(bytes).and_then(Self::try_from)
    }
}

impl TryFrom<u16> for NonZero<u16> {
    type Error = ConversionError;

    fn try_from(val: u16) -> Result<Self, Self::Error> {
        if val == 0 {
            return Err(ValueIsZero.into());
        }

        Ok(Self(val))
    }
}

impl From<NonZero<u16>> for u16 {
    fn from(val: NonZero<u16>) -> Self {
        val.get()
    }
}

impl ByteLen for NonZero<u16> {
    fn byte_len(&self) -> usize {
        self.get().byte_len()
    }
}

impl Encode for NonZero<u16> {
    fn encode(&self, buf: &mut BytesMut) {
        self.get().encode(buf)
    }
}

impl TryDecode for NonZero<u16> {
    type Error = ConversionError;

    fn try_decode(bytes: Bytes) -> Result<Self, Self::Error> {
        u16::try_decode(bytes).and_then(Self::try_from)
    }
}

impl TryFrom<u32> for NonZero<u32> {
    type Error = ConversionError;

    fn try_from(val: u32) -> Result<Self, Self::Error> {
        if val == 0 {
            return Err(ValueIsZero.into());
        }

        Ok(Self(val))
    }
}

impl From<NonZero<u32>> for u32 {
    fn from(val: NonZero<u32>) -> Self {
        val.get()
    }
}

impl ByteLen for NonZero<u32> {
    fn byte_len(&self) -> usize {
        self.get().byte_len()
    }
}

impl Encode for NonZero<u32> {
    fn encode(&self, buf: &mut BytesMut) {
        self.get().encode(buf)
    }
}

impl TryDecode for NonZero<u32> {
    type Error = ConversionError;

    fn try_decode(bytes: Bytes) -> Result<Self, Self::Error> {
        u32::try_decode(bytes).and_then(Self::try_from)
    }
}

impl TryFrom<VarSizeInt> for NonZero<VarSizeInt> {
    type Error = ConversionError;

    fn try_from(val: VarSizeInt) -> Result<Self, Self::Error> {
        if val == 0 {
            return Err(ValueIsZero.into());
        }

        Ok(Self(val))
    }
}

impl TryFrom<NonZero<VarSizeInt>> for VarSizeInt {
    type Error = ConversionError;

    fn try_from(val: NonZero<VarSizeInt>) -> Result<Self, Self::Error> {
        Ok(val.get())
    }
}

impl ByteLen for NonZero<VarSizeInt> {
    fn byte_len(&self) -> usize {
        self.get().byte_len()
    }
}

impl Encode for NonZero<VarSizeInt> {
    fn encode(&self, buf: &mut BytesMut) {
        self.get().encode(buf)
    }
}

impl TryDecode for NonZero<VarSizeInt> {
    type Error = ConversionError;

    fn try_decode(bytes: Bytes) -> Result<Self, Self::Error> {
        VarSizeInt::try_decode(bytes).and_then(Self::try_from)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Binary(pub(crate) Bytes);

impl ByteLen for Binary {
    fn byte_len(&self) -> usize {
        self.0.len() + mem::size_of::<u16>()
    }
}

impl TryDecode for Binary {
    type Error = ConversionError;

    fn try_decode(mut bytes: Bytes) -> Result<Self, Self::Error> {
        // Binary size given as two byte integer
        if mem::size_of::<u16>() > bytes.len() {
            return Err(InsufficientBufferSize.into());
        }

        let size = bytes.get_u16() as usize;
        if size > bytes.len() {
            return Err(InsufficientBufferSize.into());
        }

        Ok(Self(bytes.split_to(size)))
    }
}

impl Encode for Binary {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u16(self.0.len() as u16);
        buf.put(self.0.clone());
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BinaryRef<'a>(pub(crate) &'a [u8]);

impl<'a> ByteLen for BinaryRef<'a> {
    fn byte_len(&self) -> usize {
        mem::size_of::<u16>() + self.0.len()
    }
}

impl<'a> Encode for BinaryRef<'a> {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u16(self.0.len() as u16);
        buf.put(self.0);
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub(crate) struct Payload(pub(crate) Bytes);

impl ByteLen for Payload {
    fn byte_len(&self) -> usize {
        self.0.len()
    }
}

impl TryDecode for Payload {
    type Error = ConversionError;

    fn try_decode(bytes: Bytes) -> Result<Self, Self::Error> {
        Ok(Self(bytes))
    }
}

impl Encode for Payload {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put(self.0.clone());
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PayloadRef<'a>(pub(crate) &'a [u8]);

impl<'a> ByteLen for PayloadRef<'a> {
    fn byte_len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> Encode for PayloadRef<'a> {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put(self.0);
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UTF8String(pub(crate) Bytes);

impl ByteLen for UTF8String {
    fn byte_len(&self) -> usize {
        self.0.len() + mem::size_of::<u16>()
    }
}

impl TryDecode for UTF8String {
    type Error = ConversionError;

    fn try_decode(mut bytes: Bytes) -> Result<Self, Self::Error> {
        // UTF8String size given as two byte integer
        if mem::size_of::<u16>() > bytes.len() {
            return Err(InsufficientBufferSize.into());
        }

        let size_buf = bytes.split_to(mem::size_of::<u16>());
        let size = u16::try_decode(size_buf).unwrap() as usize;

        if size > bytes.len() {
            return Err(InsufficientBufferSize.into());
        }

        let chunk = bytes.split_to(size);
        std::str::from_utf8(&chunk)?;

        Ok(Self(chunk))
    }
}

impl Encode for UTF8String {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u16(self.0.len() as u16);
        buf.put(self.0.clone());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub(crate) struct UTF8StringRef<'a>(pub(crate) &'a str);

impl<'a> ByteLen for UTF8StringRef<'a> {
    fn byte_len(&self) -> usize {
        self.0.len() + mem::size_of::<u16>()
    }
}

impl<'a> Encode for UTF8StringRef<'a> {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u16(self.0.len() as u16);
        buf.put(self.0.as_bytes());
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UTF8StringPair(pub(crate) Bytes, pub(crate) Bytes);

impl ByteLen for UTF8StringPair {
    fn byte_len(&self) -> usize {
        2 * mem::size_of::<u16>() + self.0.len() + self.1.len()
    }
}

impl TryDecode for UTF8StringPair {
    type Error = ConversionError;

    fn try_decode(mut bytes: Bytes) -> Result<Self, Self::Error> {
        if mem::size_of::<u16>() > bytes.len() {
            return Err(InsufficientBufferSize.into());
        }

        let key_len = bytes.get_u16() as usize;
        if key_len > bytes.len() {
            return Err(InsufficientBufferSize.into());
        }

        let key = bytes.copy_to_bytes(key_len);
        std::str::from_utf8(&key)?;

        if mem::size_of::<u16>() > bytes.len() {
            return Err(InsufficientBufferSize.into());
        }

        let val_len = bytes.get_u16() as usize;

        if val_len > bytes.len() {
            return Err(InsufficientBufferSize.into());
        }

        let val = bytes.copy_to_bytes(val_len);
        std::str::from_utf8(&val)?;

        Ok(Self(key, val))
    }
}

impl Encode for UTF8StringPair {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u16(self.0.len() as u16);
        buf.put(self.0.clone());

        buf.put_u16(self.1.len() as u16);
        buf.put(self.1.clone());
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UTF8StringPairRef<'a>(pub(crate) &'a str, pub(crate) &'a str);

impl<'a> ByteLen for UTF8StringPairRef<'a> {
    fn byte_len(&self) -> usize {
        2 * mem::size_of::<u16>() + self.0.len() + self.1.len()
    }
}

impl<'a> Encode for UTF8StringPairRef<'a> {
    fn encode(&self, buf: &mut BytesMut) {
        buf.put_u16(self.0.len() as u16);
        buf.put(self.0.as_bytes());

        buf.put_u16(self.1.len() as u16);
        buf.put(self.1.as_bytes());
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod byte_len {
        use super::*;

        #[test]
        fn u8() {
            const VALUE: u8 = 17;
            assert_eq!(VALUE.byte_len(), mem::size_of::<u8>());
        }

        #[test]
        fn u16() {
            const VALUE: u16 = 4096;
            assert_eq!(VALUE.byte_len(), mem::size_of::<u16>());
        }

        #[test]
        fn u32() {
            const VALUE: u32 = 409665;
            assert_eq!(VALUE.byte_len(), mem::size_of::<u32>());
        }

        #[test]
        fn var_size_int() {
            let input: [(VarSizeInt, usize); 5] = [
                (VarSizeInt::from(127u8), 1),
                (VarSizeInt::from(128u8), 2),
                (VarSizeInt::from(16383u16), 2),
                (VarSizeInt::try_from(2097151u32).unwrap(), 3),
                (VarSizeInt::try_from(268435455u32).unwrap(), 4),
            ];

            for (var_size_int, expected_size) in input {
                assert_eq!(var_size_int.byte_len(), expected_size);
            }
        }

        #[test]
        fn binary() {
            const VALUE: [u8; 8] = [
                /* SIZE: */ 0x00, 0x06, /* DATA: */ 0x00, 0x04, 0x03, 0x76, 0x61, 0x6c,
            ];
            let binary = Binary(Bytes::from_static(&VALUE[2..]));
            assert_eq!(VALUE.len(), binary.byte_len());
        }

        #[test]
        fn binary_ref() {
            const VALUE: [u8; 8] = [
                /* SIZE: */ 0x00, 0x06, /* DATA: */ 0x00, 0x04, 0x03, 0x76, 0x61, 0x6c,
            ];
            let binary = BinaryRef(&VALUE[2..]);
            assert_eq!(VALUE.len(), binary.byte_len());
        }

        #[test]
        fn payload() {
            const VALUE: [u8; 6] = [0x00, 0x04, 0x03, 0x76, 0x61, 0x6c];
            let payload = Payload(Bytes::from_static(&VALUE));
            assert_eq!(VALUE.len(), payload.byte_len());
        }

        #[test]
        fn payload_ref() {
            const VALUE: [u8; 6] = [0x00, 0x04, 0x03, 0x76, 0x61, 0x6c];
            let payload = PayloadRef(&VALUE);
            assert_eq!(VALUE.len(), payload.byte_len());
        }

        #[test]
        fn string() {
            const INPUT: &str = "val";
            let utf8_str = UTF8String(Bytes::from_static(INPUT.as_bytes()));
            assert_eq!(INPUT.len() + 2, utf8_str.byte_len());
        }

        #[test]
        fn string_ref() {
            const INPUT: &str = "val";
            let utf8_str = UTF8StringRef(INPUT);
            assert_eq!(INPUT.len() + 2, utf8_str.byte_len());
        }

        #[test]
        fn string_pair() {
            const KEY: &str = "key";
            const VAL: &str = "val";
            let utf8_str_pair = UTF8StringPair(
                Bytes::from_static(KEY.as_bytes()),
                Bytes::from_static(VAL.as_bytes()),
            );

            assert_eq!(4 + KEY.len() + VAL.len(), utf8_str_pair.byte_len());
        }

        #[test]
        fn string_pair_ref() {
            const KEY: &str = "key";
            const VAL: &str = "val";
            let utf8_str_pair = UTF8StringPairRef(KEY, VAL);

            assert_eq!(4 + KEY.len() + VAL.len(), utf8_str_pair.byte_len());
        }
    }

    mod encode {
        use super::*;

        #[test]
        fn u8() {
            const VALUE: u8 = 17;
            let mut buf = BytesMut::new();
            VALUE.encode(&mut buf);

            assert_eq!(&[VALUE][..], &buf.split().freeze());
        }

        #[test]
        fn u16() {
            const VALUE: u16 = 4096;
            let mut buf = BytesMut::new();
            VALUE.encode(&mut buf);

            assert_eq!(&VALUE.to_be_bytes()[..], &buf.split().freeze());
        }

        #[test]
        fn u32() {
            const VALUE: u32 = 409665;
            let mut buf = BytesMut::new();
            VALUE.encode(&mut buf);

            assert_eq!(&VALUE.to_be_bytes()[..], &buf.split().freeze());
        }

        #[test]
        fn var_size_int() {
            let input: [(VarSizeInt, &[u8]); 5] = [
                (VarSizeInt::from(127u8), &[0x7f]),
                (VarSizeInt::from(128u8), &[0x80, 0x01]),
                (VarSizeInt::from(16383u16), &[0xff, 0x7f]),
                (
                    VarSizeInt::try_from(2097151u32).unwrap(),
                    &[0xff, 0xff, 0x7f],
                ),
                (
                    VarSizeInt::try_from(268435455u32).unwrap(),
                    &[0xff, 0xff, 0xff, 0x7f],
                ),
            ];

            for (var_size_int, expected_bytes) in input {
                let mut buf = BytesMut::new();
                var_size_int.encode(&mut buf);
                assert_eq!(expected_bytes, &buf.split().freeze()[..]);
            }
        }

        #[test]
        fn binary() {
            const VALUE: [u8; 8] = [
                /* SIZE: */ 0x00, 0x06, /* DATA: */ 0x00, 0x04, 0x03, 0x76, 0x61, 0x6c,
            ];
            let mut buf = BytesMut::new();
            Binary(Bytes::from_static(&VALUE[2..])).encode(&mut buf);

            assert_eq!(&VALUE[..], &buf.split().freeze());
        }

        #[test]
        fn binary_ref() {
            const VALUE: [u8; 8] = [
                /* SIZE: */ 0x00, 0x06, /* DATA: */ 0x00, 0x04, 0x03, 0x76, 0x61, 0x6c,
            ];
            let mut buf = BytesMut::new();
            BinaryRef(&VALUE[2..]).encode(&mut buf);

            assert_eq!(&VALUE[..], &buf.split().freeze());
        }

        #[test]
        fn payload() {
            const VALUE: [u8; 6] = [0x00, 0x04, 0x03, 0x76, 0x61, 0x6c];
            let mut buf = BytesMut::new();
            Payload(Bytes::from_static(&VALUE)).encode(&mut buf);

            assert_eq!(&VALUE[..], &buf.split().freeze());
        }

        #[test]
        fn payload_ref() {
            const VALUE: [u8; 6] = [0x00, 0x04, 0x03, 0x76, 0x61, 0x6c];
            let mut buf = BytesMut::new();
            PayloadRef(&VALUE).encode(&mut buf);

            assert_eq!(&VALUE[..], &buf.split().freeze());
        }

        #[test]
        fn string() {
            const INPUT: &str = "val";
            const EXPECTED_VAL: [u8; 5] = [0x00, 0x03, b'v', b'a', b'l'];

            let mut buf = BytesMut::new();
            UTF8String(Bytes::from_static(INPUT.as_bytes())).encode(&mut buf);

            assert_eq!(&EXPECTED_VAL[..], &buf.split().freeze());
        }

        #[test]
        fn string_ref() {
            const INPUT: &str = "val";
            const EXPECTED_VAL: [u8; 5] = [0x00, 0x03, b'v', b'a', b'l'];

            let mut buf = BytesMut::new();
            UTF8StringRef(INPUT).encode(&mut buf);

            assert_eq!(&EXPECTED_VAL[..], &buf.split().freeze());
        }

        #[test]
        fn string_pair() {
            const KEY: &str = "key";
            const VAL: &str = "val";
            const EXPECTED_VAL: [u8; 10] =
                [0x00, 0x03, b'k', b'e', b'y', 0x00, 0x03, b'v', b'a', b'l'];

            let mut buf = BytesMut::new();
            UTF8StringPair(
                Bytes::from_static(KEY.as_bytes()),
                Bytes::from_static(VAL.as_bytes()),
            )
            .encode(&mut buf);

            assert_eq!(&EXPECTED_VAL[..], &buf.split().freeze());
        }

        #[test]
        fn string_pair_ref() {
            const KEY: &str = "key";
            const VAL: &str = "val";
            const EXPECTED_VAL: [u8; 10] =
                [0x00, 0x03, b'k', b'e', b'y', 0x00, 0x03, b'v', b'a', b'l'];

            let mut buf = BytesMut::new();
            UTF8StringPairRef(KEY, VAL).encode(&mut buf);

            assert_eq!(&EXPECTED_VAL[..], &buf.split().freeze());
        }
    }

    mod try_decode {
        use super::*;

        #[test]
        fn u8() {
            const EXPECTED_VALUE: u8 = 73;
            const INPUT: [u8; 1] = [EXPECTED_VALUE];
            let result = u8::try_decode(Bytes::from_static(&INPUT)).unwrap();
            assert_eq!(result, EXPECTED_VALUE);
        }

        #[test]
        fn u16() {
            const EXPECTED_VALUE: u16 = 0x140;
            const INPUT: [u8; 2] = [0x1, 0x40];
            let result = u16::try_decode(Bytes::from_static(&INPUT)).unwrap();
            assert_eq!(result, EXPECTED_VALUE);
        }

        #[test]
        fn u32() {
            const EXPECTED_VALUE: u32 = 0x7d40;
            const INPUT: [u8; 4] = [0x00, 0x00, 0x7d, 0x40];
            let result = u32::try_decode(Bytes::from_static(&INPUT)).unwrap();
            assert_eq!(result, EXPECTED_VALUE);
        }

        #[test]
        fn var_size_int() {
            const INPUT: [(&[u8], usize, u32); 4] = [
                (&[0x7f], 1, 127),
                (&[0xff, 0x7f], 2, 16383),
                (&[0xff, 0xff, 0x7f], 3, 2097151),
                (&[0xff, 0xff, 0xff, 0x7f], 4, 268435455),
            ];

            for (bytes, expected_size, expected_value) in INPUT {
                let result = VarSizeInt::try_decode(Bytes::from_static(bytes));

                assert!(result.is_ok());
                assert_eq!(result.as_ref().unwrap().len(), expected_size);
                assert_eq!(result.as_ref().unwrap().value(), expected_value);
            }
        }

        #[test]
        fn var_size_int_invalid() {
            const INPUT: [&[u8]; 4] = [
                &[0xff],
                &[0xff, 0xff],
                &[0xff, 0xff, 0xff],
                &[0xff, 0xff, 0xff, 0xff],
            ];

            for bytes in INPUT {
                let result = VarSizeInt::try_decode(Bytes::from_static(bytes));
                assert!(result.is_err());
            }
        }

        #[test]
        fn binary() {
            const INPUT: [u8; 6] = [0x00, 0x04, 0x03, 0x76, 0x61, 0x6c];
            let val = Binary::try_decode(Bytes::from_static(&INPUT)).unwrap();
            assert_eq!(&val.0[..], &[0x03, 0x76, 0x61, 0x6c]);
        }

        #[test]
        fn payload() {
            const INPUT: [u8; 4] = [0x03, 0x76, 0x61, 0x6c];
            let val = Payload::try_decode(Bytes::from_static(&INPUT)).unwrap();
            assert_eq!(&val.0[..], &[0x03, 0x76, 0x61, 0x6c]);
        }

        #[test]
        fn binary_invalid_size() {
            const INPUT: [u8; 6] = [0xff, 0xff, 0x03, 0x76, 0x61, 0x6c];
            let val = Binary::try_decode(Bytes::from_static(&INPUT));
            assert!(val.is_err());
        }

        #[test]
        fn string() {
            const EXPECTED_VAL: &str = "val";
            const INPUT: [u8; 5] = [0x00, 0x03, b'v', b'a', b'l'];
            let val = UTF8String::try_decode(Bytes::from_static(&INPUT)).unwrap();
            assert_eq!(&val.0[..], EXPECTED_VAL.as_bytes());
        }

        #[test]
        fn string_invalid_size() {
            const INPUT: [u8; 5] = [0xff, 0xff, b'v', b'a', b'l'];
            let val = UTF8String::try_decode(Bytes::from_static(&INPUT));
            assert!(val.is_err());
        }

        #[test]
        fn string_pair() {
            const EXPECTED_KEY: &str = "key";
            const EXPECTED_VAL: &str = "val";
            const INPUT: [u8; 10] = [0x00, 0x03, b'k', b'e', b'y', 0x00, 0x03, b'v', b'a', b'l'];
            let result = UTF8StringPair::try_decode(Bytes::from_static(&INPUT)).unwrap();
            assert_eq!(&result.0, EXPECTED_KEY.as_bytes());
            assert_eq!(&result.1, EXPECTED_VAL.as_bytes());
        }

        #[test]
        fn utf8string_pair_invalid_size() {
            const INPUT: [u8; 10] = [0x00, 0x03, b'k', b'e', b'y', 0xff, 0xff, b'v', b'a', b'l'];
            let val = UTF8StringPair::try_decode(Bytes::from_static(&INPUT));
            assert!(val.is_err());
        }
    }

    mod conversion {
        use super::*;

        #[test]
        fn var_size_int_from_u8() {
            const INPUT: [(u8, usize); 4] =
                [(0, 1), (u8::MAX, 2), (0b10000000, 2), (0b01111111, 1)];

            for (val, expected_len) in INPUT {
                let result = VarSizeInt::from(val);

                assert_eq!(expected_len, result.len());
                assert_eq!(val as u32, result.value());
            }
        }

        #[test]
        fn var_size_int_from_u16() {
            const INPUT: [(u16, usize); 5] = [
                (0, 1),
                (u16::MAX, 3),
                (u8::MAX as u16, 2),
                (0b10000000, 2),
                (0b01111111, 1),
            ];

            for (val, expected_len) in INPUT {
                let result = VarSizeInt::from(val);

                assert_eq!(expected_len, result.len());
                assert_eq!(val as u32, result.value());
            }
        }

        #[test]
        fn var_size_int_from_u32() {
            const INPUT: [(u32, usize); 6] = [
                (0, 1),
                (VarSizeInt::MAX as u32, 4),
                (u16::MAX as u32, 3),
                (u8::MAX as u32, 2),
                (0b10000000, 2),
                (0b01111111, 1),
            ];

            for (val, expected_len) in INPUT {
                let result = VarSizeInt::try_from(val).unwrap();

                assert_eq!(expected_len, result.len());
                assert_eq!(val as u32, result.value());
            }
        }

        #[test]
        fn var_size_int_from_usize() {
            const INPUT: [(usize, usize); 6] = [
                (0, 1),
                (VarSizeInt::MAX as usize, 4),
                (u16::MAX as usize, 3),
                (u8::MAX as usize, 2),
                (0b10000000, 2),
                (0b01111111, 1),
            ];

            for (val, expected_len) in INPUT {
                let result = VarSizeInt::try_from(val).unwrap();

                assert_eq!(expected_len, result.len());
                assert_eq!(val as usize, result.value() as usize);
            }
        }

        #[test]
        fn non_zero_from_0() {
            assert!(NonZero::<u8>::try_from(0).is_err());
        }

        #[test]
        fn non_zero_from_1() {
            assert!(NonZero::<u8>::try_from(1).is_ok());
        }
    }
}
