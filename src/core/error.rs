use core::fmt;

#[derive(Debug, Clone)]
pub struct InvalidValue {}

impl fmt::Display for InvalidValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid value")
    }
}

#[derive(Debug, Clone)]
pub struct InvalidHeader {}

impl fmt::Display for InvalidHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid packet header")
    }
}

#[derive(Debug, Clone)]
pub struct InvalidPacketLength {}

impl fmt::Display for InvalidPacketLength {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid packet length")
    }
}

#[derive(Debug, Clone)]
pub struct InvalidPropertyLength {}

impl fmt::Display for InvalidPropertyLength {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid property length")
    }
}

#[derive(Debug, Clone)]
pub struct InsufficientBufferSize {}

impl fmt::Display for InsufficientBufferSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "insufficient buffer size")
    }
}

pub enum MqttError {}
