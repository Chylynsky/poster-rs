use crate::core::utils::{
    SizedProperty, ToByteBuffer, TryFromBytes, TryFromIterator, TryToByteBuffer,
};
use core::{
    convert::From,
    iter::Iterator,
    mem,
    ops::{Add, Div, Mul, Sub},
    str,
};

use std::{string::String, vec::Vec};

#[allow(clippy::enum_variant_names)]
#[derive(Copy, Clone, PartialEq, Debug, Eq)]
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
    pub(crate) const MIN: usize = 0;

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

    #[allow(clippy::wrong_self_convention)]
    fn to_byte_buffer_unchecked<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.len()];

        match self.0 {
            VarSizeIntState::SingleByte(val) => result.copy_from_slice(&[val]),
            VarSizeIntState::TwoByte(val) => {
                result.copy_from_slice(&[((val >> 7) | 0x80) as u8, (val & 0x7f) as u8])
            }
            VarSizeIntState::ThreeByte(val) => result.copy_from_slice(&[
                (((val >> 14) & 0x7f) | 0x80) as u8,
                (((val >> 7) & 0x7f) | 0x80) as u8,
                (val & 0x7f) as u8,
            ]),
            VarSizeIntState::FourByte(val) => result.copy_from_slice(&[
                (((val >> 21) & 0x7f) | 0x80) as u8,
                (((val >> 14) & 0x7f) | 0x80) as u8,
                (((val >> 7) & 0x7f) | 0x80) as u8,
                (val & 0x7f) as u8,
            ]),
        }

        result
    }
}

impl Default for VarSizeInt {
    fn default() -> Self {
        Self(VarSizeIntState::SingleByte(0))
    }
}

impl SizedProperty for VarSizeInt {
    fn property_len(&self) -> usize {
        self.len()
    }
}

impl TryFromIterator<u8> for VarSizeInt {
    fn try_from_iter<Iter>(iter: Iter) -> Option<Self>
    where
        Iter: Iterator<Item = u8>,
    {
        let mut size = 0usize;
        let mut end = false;

        let result = iter
            .enumerate()
            .take_while(|(idx, byte)| {
                if end || *idx == 4 {
                    return false;
                } else if (byte >> 0x7) == 0x0 {
                    end = true;
                }
                true
            })
            .map(|(idx, byte)| {
                size = idx + 1;
                byte as u32
            })
            .fold(0u32, |acc, val| (acc << 7) | (val & 0x7f));

        if !end || result as usize > Self::MAX {
            return None;
        }

        match size {
            1 => Some(Self(VarSizeIntState::SingleByte(result as u8))),
            2 => Some(Self(VarSizeIntState::TwoByte(result as u16))),
            3 => Some(Self(VarSizeIntState::ThreeByte(result as u32))),
            4 => Some(Self(VarSizeIntState::FourByte(result as u32))),
            _ => None,
        }
    }
}

impl TryFromBytes for VarSizeInt {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        let iter = bytes.iter().copied();
        VarSizeInt::try_from_iter(iter)
    }
}

impl TryToByteBuffer for VarSizeInt {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        if self.len() > buf.len() {
            return None;
        }

        Some(self.to_byte_buffer_unchecked(buf))
    }
}

impl ToByteBuffer for VarSizeInt {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        self.to_byte_buffer_unchecked(buf)
    }
}

impl Add for VarSizeInt {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::from(self.value() + rhs.value())
    }
}

impl Sub for VarSizeInt {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::from(self.value() - rhs.value())
    }
}

impl Mul for VarSizeInt {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self::from(self.value() * rhs.value())
    }
}

impl Div for VarSizeInt {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self::from(self.value() / rhs.value())
    }
}

impl From<u8> for VarSizeInt {
    fn from(val: u8) -> Self {
        if val >> 7 == 0 {
            return Self(VarSizeIntState::SingleByte(val));
        }

        Self(VarSizeIntState::TwoByte(val as u16))
    }
}

impl From<u16> for VarSizeInt {
    fn from(val: u16) -> Self {
        if val >> 7 == 0 {
            return Self(VarSizeIntState::SingleByte(val as u8));
        } else if val >> 15 == 0 {
            return Self(VarSizeIntState::TwoByte(val));
        }

        Self(VarSizeIntState::ThreeByte(val as u32))
    }
}

impl From<u32> for VarSizeInt {
    fn from(val: u32) -> Self {
        if val >> 7 == 0 {
            return Self(VarSizeIntState::SingleByte(val as u8));
        } else if val >> 15 == 0 {
            return Self(VarSizeIntState::TwoByte(val as u16));
        } else if val >> 23 == 0 {
            return Self(VarSizeIntState::ThreeByte(val));
        }

        assert!(val <= Self::MAX as u32);
        Self(VarSizeIntState::FourByte(val))
    }
}

impl From<usize> for VarSizeInt {
    fn from(val: usize) -> Self {
        if val >> 7 == 0 {
            return Self(VarSizeIntState::SingleByte(val as u8));
        } else if val >> 15 == 0 {
            return Self(VarSizeIntState::TwoByte(val as u16));
        } else if val >> 23 == 0 {
            return Self(VarSizeIntState::ThreeByte(val as u32));
        }

        assert!(val <= Self::MAX);
        Self(VarSizeIntState::FourByte(val as u32))
    }
}

impl From<VarSizeInt> for u8 {
    fn from(val: VarSizeInt) -> Self {
        match val.0 {
            VarSizeIntState::SingleByte(val) => val as u8,
            VarSizeIntState::TwoByte(val) => {
                assert!(val <= 0xff);
                val as u8
            }
            _ => panic!(),
        }
    }
}

impl From<VarSizeInt> for u16 {
    fn from(val: VarSizeInt) -> Self {
        match val.0 {
            VarSizeIntState::SingleByte(val) => val as u16,
            VarSizeIntState::TwoByte(val) => val as u16,
            VarSizeIntState::ThreeByte(val) => {
                assert!(val <= 0xffff);
                val as u16
            }
            _ => panic!(),
        }
    }
}

impl From<VarSizeInt> for u32 {
    fn from(val: VarSizeInt) -> Self {
        match val.0 {
            VarSizeIntState::SingleByte(val) => val as u32,
            VarSizeIntState::TwoByte(val) => val as u32,
            VarSizeIntState::ThreeByte(val) => val,
            VarSizeIntState::FourByte(val) => {
                assert!(val as usize <= VarSizeInt::MAX);
                val
            }
        }
    }
}

impl From<VarSizeInt> for usize {
    fn from(val: VarSizeInt) -> Self {
        match val.0 {
            VarSizeIntState::SingleByte(val) => val as usize,
            VarSizeIntState::TwoByte(val) => val as usize,
            VarSizeIntState::ThreeByte(val) => val as usize,
            VarSizeIntState::FourByte(val) => {
                assert!(val as usize <= VarSizeInt::MAX);
                val as usize
            }
        }
    }
}

impl SizedProperty for u8 {
    fn property_len(&self) -> usize {
        mem::size_of::<Self>()
    }
}

impl TryFromBytes for u8 {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        bytes.get(0).copied()
    }
}

impl ToByteBuffer for u8 {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        buf[0] = *self;
        &buf[0..1]
    }
}

impl TryToByteBuffer for u8 {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        *buf.get_mut(0)? = *self;
        Some(&buf[0..1])
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum QoS {
    AtMostOnce = 0,
    AtLeastOnce = 1,
    ExactlyOnce = 2,
}

impl QoS {
    pub fn try_from(val: u8) -> Option<Self> {
        match val {
            0 => Some(QoS::AtMostOnce),
            1 => Some(QoS::AtLeastOnce),
            2 => Some(QoS::ExactlyOnce),
            _ => None,
        }
    }
}

impl Default for QoS {
    fn default() -> Self {
        QoS::AtMostOnce
    }
}

impl SizedProperty for QoS {
    fn property_len(&self) -> usize {
        mem::size_of::<u8>()
    }
}

impl TryFromBytes for QoS {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        Self::try_from(*bytes.get(0)?)
    }
}

impl ToByteBuffer for QoS {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        buf[0] = *self as u8;
        &buf[0..1]
    }
}

impl TryToByteBuffer for QoS {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        (*self as u8).try_to_byte_buffer(buf)
    }
}

impl SizedProperty for bool {
    fn property_len(&self) -> usize {
        mem::size_of::<u8>()
    }
}

impl TryFromBytes for bool {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes.iter().copied().next() {
            Some(0u8) => Some(false),
            Some(1u8) => Some(true),
            _ => None,
        }
    }
}

impl ToByteBuffer for bool {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        (*self as u8).to_byte_buffer(buf)
    }
}

impl TryToByteBuffer for bool {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        (*self as u8).try_to_byte_buffer(buf)
    }
}

impl SizedProperty for u16 {
    fn property_len(&self) -> usize {
        mem::size_of::<Self>()
    }
}

impl TryFromBytes for u16 {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        bytes
            .iter()
            .take(mem::size_of::<u16>())
            .map(|&value| value as u16)
            .reduce(|result, tmp| result << 8 | tmp)
    }
}

impl ToByteBuffer for u16 {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        result.copy_from_slice(&self.to_be_bytes());
        result
    }
}

impl TryToByteBuffer for u16 {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        result.copy_from_slice(&self.to_be_bytes());
        Some(result)
    }
}

impl SizedProperty for u32 {
    fn property_len(&self) -> usize {
        mem::size_of::<Self>()
    }
}

impl ToByteBuffer for u32 {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        result.copy_from_slice(&self.to_be_bytes());
        result
    }
}

impl TryToByteBuffer for u32 {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        result.copy_from_slice(&self.to_be_bytes());
        Some(result)
    }
}

impl TryFromBytes for u32 {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        bytes
            .iter()
            .take(mem::size_of::<u32>())
            .map(|&value| value as u32)
            .reduce(|result, tmp| result << 8 | tmp)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NonZero<T>(T)
where
    T: Copy;

impl<T> NonZero<T>
where
    T: Copy + std::cmp::PartialEq<i32>,
{
    pub fn try_from(val: T) -> Option<Self> {
        if val == 0 {
            return None;
        }

        Some(Self(val))
    }

    pub fn value(&self) -> T {
        self.0
    }
}

impl From<u8> for NonZero<u8> {
    fn from(val: u8) -> Self {
        assert!(val != 0);
        Self(val)
    }
}

impl From<NonZero<u8>> for u8 {
    fn from(val: NonZero<u8>) -> Self {
        val.0
    }
}

impl SizedProperty for NonZero<u8> {
    fn property_len(&self) -> usize {
        self.0.property_len()
    }
}

impl ToByteBuffer for NonZero<u8> {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        self.0.to_byte_buffer(buf)
    }
}

impl TryToByteBuffer for NonZero<u8> {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        self.0.try_to_byte_buffer(buf)
    }
}

impl TryFromBytes for NonZero<u8> {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        u8::try_from_bytes(bytes).and_then(|val| {
            if val == 0 {
                return None;
            }

            return Some(NonZero(val));
        })
    }
}

impl From<u16> for NonZero<u16> {
    fn from(val: u16) -> Self {
        assert!(val != 0);
        Self(val)
    }
}

impl From<NonZero<u16>> for u16 {
    fn from(val: NonZero<u16>) -> Self {
        val.0
    }
}

impl SizedProperty for NonZero<u16> {
    fn property_len(&self) -> usize {
        self.0.property_len()
    }
}

impl ToByteBuffer for NonZero<u16> {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        self.0.to_byte_buffer(buf)
    }
}

impl TryToByteBuffer for NonZero<u16> {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        self.0.try_to_byte_buffer(buf)
    }
}

impl TryFromBytes for NonZero<u16> {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        u16::try_from_bytes(bytes).and_then(|val| {
            if val == 0 {
                return None;
            }

            return Some(NonZero(val));
        })
    }
}

impl From<u32> for NonZero<u32> {
    fn from(val: u32) -> Self {
        assert!(val != 0);
        Self(val)
    }
}

impl From<NonZero<u32>> for u32 {
    fn from(val: NonZero<u32>) -> Self {
        val.0
    }
}

impl SizedProperty for NonZero<u32> {
    fn property_len(&self) -> usize {
        self.0.property_len()
    }
}

impl ToByteBuffer for NonZero<u32> {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        self.0.to_byte_buffer(buf)
    }
}

impl TryToByteBuffer for NonZero<u32> {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        self.0.try_to_byte_buffer(buf)
    }
}

impl TryFromBytes for NonZero<u32> {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        u32::try_from_bytes(bytes).and_then(|val| {
            if val == 0 {
                return None;
            }

            return Some(NonZero(val));
        })
    }
}

impl From<VarSizeInt> for NonZero<VarSizeInt> {
    fn from(val: VarSizeInt) -> Self {
        assert!(val.value() != 0);
        Self(val)
    }
}

impl From<NonZero<VarSizeInt>> for VarSizeInt {
    fn from(val: NonZero<VarSizeInt>) -> Self {
        val.0
    }
}

impl SizedProperty for NonZero<VarSizeInt> {
    fn property_len(&self) -> usize {
        self.0.property_len()
    }
}

impl ToByteBuffer for NonZero<VarSizeInt> {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        self.0.to_byte_buffer(buf)
    }
}

impl TryToByteBuffer for NonZero<VarSizeInt> {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        self.0.try_to_byte_buffer(buf)
    }
}

impl TryFromBytes for NonZero<VarSizeInt> {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        VarSizeInt::try_from_bytes(bytes).and_then(|val| {
            if val.value() == 0 {
                return None;
            }

            return Some(NonZero(val));
        })
    }
}

pub(crate) type Binary = Vec<u8>;

impl SizedProperty for Binary {
    fn property_len(&self) -> usize {
        self.len() + mem::size_of::<u16>()
    }
}

impl TryFromBytes for Binary {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        // Binary size given as two byte integer
        if mem::size_of::<u16>() > bytes.len() {
            return None;
        }

        let (size_buf, remaining) = bytes.split_at(mem::size_of::<u16>());
        let size = size_buf
            .iter()
            .map(|&value| value as usize)
            .reduce(|result, tmp| result << 8 | tmp)?;

        Some(Vec::from(remaining.get(0..size)?))
    }
}

impl ToByteBuffer for Binary {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        result.copy_from_slice(&[&(self.len() as u16).to_be_bytes()[..], self].concat());
        result
    }
}

impl TryToByteBuffer for Binary {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        result.copy_from_slice(&[&(self.len() as u16).to_be_bytes()[..], self].concat());
        Some(result)
    }
}

pub(crate) type BinaryRef<'a> = &'a [u8];

impl<'a> SizedProperty for BinaryRef<'a> {
    fn property_len(&self) -> usize {
        self.len() + mem::size_of::<u16>()
    }
}

impl<'a> ToByteBuffer for BinaryRef<'a> {
    fn to_byte_buffer<'b>(&self, buf: &'b mut [u8]) -> &'b [u8] {
        let result = &mut buf[0..self.property_len()];
        result.copy_from_slice(&[&(self.len() as u16).to_be_bytes()[..], self].concat());
        result
    }
}

impl<'a> TryToByteBuffer for BinaryRef<'a> {
    fn try_to_byte_buffer<'b>(&self, buf: &'b mut [u8]) -> Option<&'b [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        result.copy_from_slice(&[&(self.len() as u16).to_be_bytes()[..], self].concat());
        Some(result)
    }
}

impl SizedProperty for String {
    fn property_len(&self) -> usize {
        self.len() + mem::size_of::<u16>()
    }
}

impl TryFromBytes for String {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        // Binary size given as two byte integer
        if mem::size_of::<u16>() > bytes.len() {
            return None;
        }

        let (size_buf, remaining) = bytes.split_at(mem::size_of::<u16>());
        let size = size_buf
            .iter()
            .map(|&value| value as usize)
            .reduce(|result, tmp| result << 8 | tmp)?;

        String::from_utf8(Vec::from(remaining.get(0..size)?)).ok()
    }
}

impl ToByteBuffer for String {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        result.copy_from_slice(&[&(self.len() as u16).to_be_bytes()[..], self.as_bytes()].concat());
        result
    }
}

impl TryToByteBuffer for String {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        result.copy_from_slice(&[&(self.len() as u16).to_be_bytes()[..], self.as_bytes()].concat());
        Some(result)
    }
}

pub(crate) type StringRef<'a> = &'a str;

impl<'a> SizedProperty for StringRef<'a> {
    fn property_len(&self) -> usize {
        self.len() + mem::size_of::<u16>()
    }
}

impl<'a> ToByteBuffer for StringRef<'a> {
    fn to_byte_buffer<'b>(&self, buf: &'b mut [u8]) -> &'b [u8] {
        let result = &mut buf[0..self.property_len()];
        result.copy_from_slice(&[&(self.len() as u16).to_be_bytes()[..], self.as_bytes()].concat());
        result
    }
}

impl<'a> TryToByteBuffer for StringRef<'a> {
    fn try_to_byte_buffer<'b>(&self, buf: &'b mut [u8]) -> Option<&'b [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        result.copy_from_slice(&[&(self.len() as u16).to_be_bytes()[..], self.as_bytes()].concat());
        Some(result)
    }
}

pub(crate) type StringPair = (String, String);

impl SizedProperty for StringPair {
    fn property_len(&self) -> usize {
        self.0.property_len() + self.1.property_len()
    }
}

impl TryFromBytes for StringPair {
    fn try_from_bytes(bytes: &[u8]) -> Option<Self> {
        if mem::size_of::<u16>() > bytes.len() {
            return None;
        }

        let (key_size_buf, remaining) = bytes.split_at(mem::size_of::<u16>());
        let key_size = key_size_buf
            .iter()
            .map(|&value| value as usize)
            .reduce(|result, tmp| result << 8 | tmp)?;

        if key_size > remaining.len() {
            return None;
        }

        let (key_buf, remaining) = remaining.split_at(key_size);
        if mem::size_of::<u16>() > remaining.len() {
            return None;
        }

        let (value_size_buf, remaining) = remaining.split_at(mem::size_of::<u16>());
        let value_size = value_size_buf
            .iter()
            .map(|&value| value as usize)
            .reduce(|result, tmp| result << 8 | tmp)?;

        if value_size > remaining.len() {
            return None;
        }

        let (value_buf, _) = remaining.split_at(value_size);

        Some((
            String::from_utf8(Vec::from(&key_buf[0..key_size])).ok()?,
            String::from_utf8(Vec::from(&value_buf[0..value_size])).ok()?,
        ))
    }
}

impl ToByteBuffer for StringPair {
    fn to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> &'a [u8] {
        let result = &mut buf[0..self.property_len()];
        let (key, val) = &self;

        result.copy_from_slice(
            &[
                &(key.len() as u16).to_be_bytes()[..],
                key.as_bytes(),
                &(val.len() as u16).to_be_bytes()[..],
                val.as_bytes(),
            ]
            .concat(),
        );
        result
    }
}

impl TryToByteBuffer for StringPair {
    fn try_to_byte_buffer<'a>(&self, buf: &'a mut [u8]) -> Option<&'a [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        let (key, val) = &self;

        result.copy_from_slice(
            &[
                &(key.len() as u16).to_be_bytes()[..],
                key.as_bytes(),
                &(val.len() as u16).to_be_bytes()[..],
                val.as_bytes(),
            ]
            .concat(),
        );
        Some(result)
    }
}

pub(crate) type StringPairRef<'a> = (StringRef<'a>, StringRef<'a>);

impl<'a> SizedProperty for StringPairRef<'a> {
    fn property_len(&self) -> usize {
        self.0.property_len() + self.1.property_len()
    }
}

impl<'a> ToByteBuffer for StringPairRef<'a> {
    fn to_byte_buffer<'b>(&self, buf: &'b mut [u8]) -> &'b [u8] {
        let result = &mut buf[0..self.property_len()];
        let (key, val) = &self;

        result.copy_from_slice(
            &[
                &(key.len() as u16).to_be_bytes()[..],
                key.as_bytes(),
                &(val.len() as u16).to_be_bytes()[..],
                val.as_bytes(),
            ]
            .concat(),
        );
        result
    }
}

impl<'a> TryToByteBuffer for StringPairRef<'a> {
    fn try_to_byte_buffer<'b>(&self, buf: &'b mut [u8]) -> Option<&'b [u8]> {
        let result = buf.get_mut(0..self.property_len())?;
        let (key, val) = &self;

        result.copy_from_slice(
            &[
                &(key.len() as u16).to_be_bytes()[..],
                key.as_bytes(),
                &(val.len() as u16).to_be_bytes()[..],
                val.as_bytes(),
            ]
            .concat(),
        );
        Some(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod from_bytes {
        use super::*;

        #[test]
        fn byte() {
            const EXPECTED_VALUE: u8 = 73;
            const INPUT: [u8; 1] = [EXPECTED_VALUE];
            let result = u8::try_from_bytes(&INPUT).unwrap();
            assert_eq!(result, EXPECTED_VALUE);
        }

        #[test]
        fn two_byte_int() {
            const EXPECTED_VALUE: u16 = 0x140;
            const INPUT: [u8; 2] = [0x1, 0x40];
            let result = u16::try_from_bytes(&INPUT).unwrap();
            assert_eq!(result, EXPECTED_VALUE);
        }

        #[test]
        fn four_byte_int() {
            const EXPECTED_VALUE: u32 = 0x7d40;
            const INPUT: [u8; 4] = [0x00, 0x00, 0x7d, 0x40];
            let result = u32::try_from_bytes(&INPUT).unwrap();
            assert_eq!(result, EXPECTED_VALUE);
        }

        #[test]
        fn binary() {
            const INPUT: [u8; 6] = [0x00, 0x04, 0x03, 0x76, 0x61, 0x6c];
            let val = Binary::try_from_bytes(&INPUT).unwrap();
            assert_eq!(val, [0x03, 0x76, 0x61, 0x6c]);
        }

        #[test]
        fn binary_invalid_size() {
            const INPUT: [u8; 6] = [0xff, 0xff, 0x03, 0x76, 0x61, 0x6c];
            let val = Binary::try_from_bytes(&INPUT);
            assert!(val.is_none());
        }

        #[test]
        fn utf8string() {
            const EXPECTED_VAL: &str = "val";
            const INPUT: [u8; 5] = [0x00, 0x03, b'v', b'a', b'l'];
            let val = String::try_from_bytes(&INPUT).unwrap();
            assert_eq!(val, EXPECTED_VAL);
        }

        #[test]
        fn utf8string_invalid_size() {
            const INPUT: [u8; 5] = [0xff, 0xff, b'v', b'a', b'l'];
            let val = String::try_from_bytes(&INPUT);
            assert!(val.is_none());
        }

        #[test]
        fn utf8string_pair() {
            const EXPECTED_KEY: &str = "key";
            const EXPECTED_VAL: &str = "val";
            const INPUT: [u8; 10] = [0x00, 0x03, b'k', b'e', b'y', 0x00, 0x03, b'v', b'a', b'l'];
            let (key, val) = StringPair::try_from_bytes(&INPUT).unwrap();
            assert_eq!(key, EXPECTED_KEY);
            assert_eq!(val, EXPECTED_VAL);
        }

        #[test]
        fn utf8string_pair_invalid_size() {
            const INPUT: [u8; 10] = [0x00, 0x03, b'k', b'e', b'y', 0xff, 0xff, b'v', b'a', b'l'];
            let val = StringPair::try_from_bytes(&INPUT);
            assert!(val.is_none());
        }
    }

    mod var_size_int {
        use super::*;

        #[test]
        fn from_u8() {
            const INPUT: [(u8, usize); 4] =
                [(0, 1), (u8::MAX, 2), (0b10000000, 2), (0b01111111, 1)];

            for (val, expected_len) in INPUT {
                let result = VarSizeInt::from(val);

                assert_eq!(expected_len, result.len());
                assert_eq!(val, result.into());
            }
        }

        #[test]
        fn from_u16() {
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
                assert_eq!(val, result.into());
            }
        }

        #[test]
        fn from_u32() {
            const INPUT: [(u32, usize); 6] = [
                (0, 1),
                (VarSizeInt::MAX as u32, 4),
                (u16::MAX as u32, 3),
                (u8::MAX as u32, 2),
                (0b10000000, 2),
                (0b01111111, 1),
            ];

            for (val, expected_len) in INPUT {
                let result = VarSizeInt::from(val);

                assert_eq!(expected_len, result.len());
                assert_eq!(val, result.into());
            }
        }

        #[test]
        fn from_iter() {
            const INPUT: [(&[u8], usize, u32); 4] = [
                (&[0x7f], 1, 127),
                (&[0xff, 0x7f], 2, 16383),
                (&[0xff, 0xff, 0x7f], 3, 2097151),
                (&[0xff, 0xff, 0xff, 0x7f], 4, 268435455),
            ];

            for (bytes, expected_size, expected_value) in INPUT {
                let result = VarSizeInt::try_from_iter(bytes.iter().copied());

                assert!(result.is_some());
                assert_eq!(result.unwrap().len(), expected_size);
                assert_eq!(result.unwrap().value(), expected_value);
            }
        }

        #[test]
        fn from_iter_invalid() {
            const INPUT: [&[u8]; 4] = [
                &[0xff],
                &[0xff, 0xff],
                &[0xff, 0xff, 0xff],
                &[0xff, 0xff, 0xff, 0xff],
            ];

            for bytes in INPUT {
                let result = VarSizeInt::try_from_iter(bytes.iter().copied());
                assert!(result.is_none());
            }
        }

        #[test]
        fn to_byte_buffer() {
            let input: [(VarSizeInt, &[u8]); 4] = [
                (VarSizeInt::from(127u8), &[0x7f]),
                (VarSizeInt::from(16383u16), &[0xff, 0x7f]),
                (VarSizeInt::from(2097151u32), &[0xff, 0xff, 0x7f]),
                (VarSizeInt::from(268435455u32), &[0xff, 0xff, 0xff, 0x7f]),
            ];

            for (var_size_int, expected_bytes) in input {
                let mut buf = [0u8; 4];
                let result = var_size_int.to_byte_buffer(&mut buf);
                assert_eq!(result, expected_bytes);
            }
        }

        #[test]
        #[should_panic]
        fn to_byte_buffer_invalid() {
            let input = VarSizeInt::from(268435455u32);
            let mut buf = [0u8; 2];
            input.to_byte_buffer(&mut buf);
        }

        #[test]
        fn try_to_byte_buffer_invalid() {
            let input = VarSizeInt::from(268435455u32);
            let mut buf = [0u8; 2];
            let result = input.try_to_byte_buffer(&mut buf);
            assert!(result.is_none());
        }
    }

    mod non_zero {
        use super::*;

        #[test]
        #[should_panic]
        fn from_zero() {
            let _ = NonZero::<u8>::from(0);
        }

        #[test]
        fn from() {
            let _ = NonZero::<u8>::from(1);
        }
    }
}
