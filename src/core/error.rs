use core::fmt;
use derive_builder::UninitializedFieldError;
use std::{error::Error, str::Utf8Error};

/// Invalid value was supplied.
///
#[derive(Debug, Clone, Copy)]
pub struct InvalidValue;

impl fmt::Display for InvalidValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid value")
    }
}

impl Error for InvalidValue {}

/// Unaccepted value `0` was supplied.
///
#[derive(Debug, Clone, Copy)]
pub struct ValueIsZero;

impl fmt::Display for ValueIsZero {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "value must be other than 0")
    }
}

impl Error for ValueIsZero {}

/// Value exceedes the allowed maximum.
///
#[derive(Debug, Clone, Copy)]
pub struct ValueExceedesMaximum;

impl fmt::Display for ValueExceedesMaximum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "value exceedes maximum")
    }
}

impl Error for ValueExceedesMaximum {}

/// Invalid byte encoding was found.
///
#[derive(Debug, Clone, Copy)]
pub struct InvalidEncoding;

impl fmt::Display for InvalidEncoding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid encoding")
    }
}

impl Error for InvalidEncoding {}

/// Size of the supplied buffer is too small.
///
#[derive(Debug, Clone, Copy)]
pub struct InsufficientBufferSize;

impl fmt::Display for InsufficientBufferSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "insufficient buffer size")
    }
}

impl Error for InsufficientBufferSize {}

/// General error type for conversion errors.
///
#[derive(Debug, Clone)]
pub enum ConversionError {
    /// See [InvalidValue].
    ///
    InvalidValue(InvalidValue),

    /// See [ValueIsZero].
    ///
    ValueIsZero(ValueIsZero),

    /// See [ValueExceedesMaximum].
    ///
    ValueExceedesMaximum(ValueExceedesMaximum),

    /// See [InvalidEncoding].
    ///
    InvalidEncoding(InvalidEncoding),

    /// See [Utf8Error].
    ///
    Utf8Error(Utf8Error),

    /// See [InsufficientBufferSize].
    ///
    InsufficientBufferSize(InsufficientBufferSize),
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidValue(err) => write!(
                f,
                "{{ \"type\": \"ConversionError\", \"message\": \"{}\" }}",
                err
            ),
            Self::ValueIsZero(err) => write!(
                f,
                "{{ \"type\": \"ConversionError\", \"message\": \"{}\" }}",
                err
            ),
            Self::ValueExceedesMaximum(err) => write!(
                f,
                "{{ \"type\": \"ConversionError\", \"message\": \"{}\" }}",
                err
            ),
            Self::InvalidEncoding(err) => write!(
                f,
                "{{ \"type\": \"ConversionError\", \"message\": \"{}\" }}",
                err
            ),
            Self::Utf8Error(err) => write!(
                f,
                "{{ \"type\": \"ConversionError\", \"message\": \"{}\" }}",
                err
            ),
            Self::InsufficientBufferSize(err) => write!(
                f,
                "{{ \"type\": \"ConversionError\", \"message\": \"{}\" }}",
                err
            ),
        }
    }
}

impl Error for ConversionError {}

impl From<InvalidValue> for ConversionError {
    fn from(err: InvalidValue) -> Self {
        Self::InvalidValue(err)
    }
}

impl From<ValueIsZero> for ConversionError {
    fn from(err: ValueIsZero) -> Self {
        Self::ValueIsZero(err)
    }
}

impl From<ValueExceedesMaximum> for ConversionError {
    fn from(err: ValueExceedesMaximum) -> Self {
        Self::ValueExceedesMaximum(err)
    }
}

impl From<InvalidEncoding> for ConversionError {
    fn from(err: InvalidEncoding) -> Self {
        Self::InvalidEncoding(err)
    }
}

impl From<Utf8Error> for ConversionError {
    fn from(err: Utf8Error) -> Self {
        Self::Utf8Error(err)
    }
}

impl From<InsufficientBufferSize> for ConversionError {
    fn from(err: InsufficientBufferSize) -> Self {
        Self::InsufficientBufferSize(err)
    }
}

/// Invalid property identifier found in an incoming packet.
///
#[derive(Debug, Clone, Copy)]
pub struct InvalidPropertyId;

impl fmt::Display for InvalidPropertyId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid property identifier")
    }
}

impl Error for InvalidPropertyId {}

/// General error type for property errors.
///
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub enum PropertyError {
    ConversionError(ConversionError),
    InvalidPropertyId(InvalidPropertyId),
}

impl fmt::Display for PropertyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ConversionError(err) => write!(f, "{}", err),
            Self::InvalidPropertyId(err) => write!(
                f,
                "{{ \"type\": \"PropertyError\", \"message\": \"{}\" }}",
                err
            ),
        }
    }
}

impl Error for PropertyError {}

impl From<ConversionError> for PropertyError {
    fn from(err: ConversionError) -> Self {
        Self::ConversionError(err)
    }
}

impl From<InvalidPropertyId> for PropertyError {
    fn from(err: InvalidPropertyId) -> Self {
        Self::InvalidPropertyId(err)
    }
}

/// Found property that is not valid for the incoming packet.
///
#[derive(Debug, Clone, Copy)]
pub struct UnexpectedProperty;

impl fmt::Display for UnexpectedProperty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unexpected property")
    }
}

impl Error for UnexpectedProperty {}

/// Header of the incoming packet is invalid.
///
#[derive(Debug, Clone, Copy)]
pub struct InvalidPacketHeader;

impl fmt::Display for InvalidPacketHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid packet header")
    }
}

impl Error for InvalidPacketHeader {}

/// Size of the incoming packet is not valid.
///
#[derive(Debug, Clone, Copy)]
pub struct InvalidPacketSize;

impl fmt::Display for InvalidPacketSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid packet size")
    }
}

impl Error for InvalidPacketSize {}

/// Declared propery length of the incoming packet is not valid.
///
#[derive(Debug, Clone, Copy)]
pub struct InvalidPropertyLength;

impl fmt::Display for InvalidPropertyLength {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid property length")
    }
}

impl Error for InvalidPropertyLength {}

/// Mandatory property is missing in the packet.
///
#[derive(Debug, Clone, Copy)]
pub struct MandatoryPropertyMissing;

impl fmt::Display for MandatoryPropertyMissing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "mandatory property missing")
    }
}

impl Error for MandatoryPropertyMissing {}

/// General error type for the packet codec.
///
#[allow(missing_docs)]
#[derive(Debug, Clone)]
pub enum CodecError {
    ConversionError(ConversionError),
    PropertyError(PropertyError),
    UnexpectedProperty(UnexpectedProperty),
    InvalidPacketHeader(InvalidPacketHeader),
    InvalidPacketSize(InvalidPacketSize),
    InvalidPropertyLength(InvalidPropertyLength),
    InsufficientBufferSize(InsufficientBufferSize),
    MandatoryPropertyMissing(MandatoryPropertyMissing),
}

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ConversionError(err) => write!(f, "{}", err),
            Self::PropertyError(err) => write!(f, " {}", err),
            Self::UnexpectedProperty(err) => write!(
                f,
                "{{ \"type\": \"CodecError\", \"message\": \"{}\" }}",
                err
            ),
            Self::InvalidPacketHeader(err) => write!(
                f,
                "{{ \"type\": \"CodecError\", \"message\": \"{}\" }}",
                err
            ),
            Self::InvalidPacketSize(err) => write!(
                f,
                "{{ \"type\": \"CodecError\", \"message\": \"{}\" }}",
                err
            ),
            Self::InvalidPropertyLength(err) => write!(
                f,
                "{{ \"type\": \"CodecError\", \"message\": \"{}\" }}",
                err
            ),
            Self::InsufficientBufferSize(err) => write!(
                f,
                "{{ \"type\": \"CodecError\", \"message\": \"{}\" }}",
                err
            ),
            Self::MandatoryPropertyMissing(err) => write!(
                f,
                "{{ \"type\": \"CodecError\", \"message\": \"{}\" }}",
                err
            ),
        }
    }
}

impl Error for CodecError {}

impl From<ConversionError> for CodecError {
    fn from(err: ConversionError) -> Self {
        Self::PropertyError(err.into())
    }
}

impl From<PropertyError> for CodecError {
    fn from(err: PropertyError) -> Self {
        Self::PropertyError(err)
    }
}

impl From<UnexpectedProperty> for CodecError {
    fn from(err: UnexpectedProperty) -> Self {
        Self::UnexpectedProperty(err)
    }
}

impl From<InvalidPacketHeader> for CodecError {
    fn from(err: InvalidPacketHeader) -> Self {
        Self::InvalidPacketHeader(err)
    }
}

impl From<InvalidPacketSize> for CodecError {
    fn from(err: InvalidPacketSize) -> Self {
        Self::InvalidPacketSize(err)
    }
}

impl From<InvalidPropertyLength> for CodecError {
    fn from(err: InvalidPropertyLength) -> Self {
        Self::InvalidPropertyLength(err)
    }
}

impl From<InsufficientBufferSize> for CodecError {
    fn from(err: InsufficientBufferSize) -> Self {
        Self::InsufficientBufferSize(err)
    }
}

impl From<MandatoryPropertyMissing> for CodecError {
    fn from(err: MandatoryPropertyMissing) -> Self {
        Self::MandatoryPropertyMissing(err)
    }
}

impl From<UninitializedFieldError> for CodecError {
    fn from(_: UninitializedFieldError) -> CodecError {
        MandatoryPropertyMissing.into()
    }
}
