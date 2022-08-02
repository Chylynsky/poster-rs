use std::fmt;

#[derive(Debug, Clone)]
struct InvalidHeader {}

impl fmt::Display for InvalidHeader {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid packet header")
    }
}

#[derive(Debug, Clone)]
struct InvalidPacketLength {}

impl fmt::Display for InvalidPacketLength {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid packet length")
    }
}

#[derive(Debug, Clone)]
struct InvalidPropertyLength {}

impl fmt::Display for InvalidPropertyLength {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid property length")
    }
}

#[derive(Debug, Clone)]
struct InsufficientBufferSize {}

impl fmt::Display for InsufficientBufferSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "insufficient buffer size")
    }
}
