use super::*;
use core::fmt;

/// The integral type used in the Odin protocol and the PIT format.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct OdinInt {
    pub inner: u32,
}

impl OdinInt {
    /// Convert to the wire representation.
    pub fn to_wire(&self) -> [u8; 4] {
        return u32::to_le_bytes(self.inner);
    }

    /// Construct from the wire representation.
    pub fn from_wire(data: [u8; 4]) -> OdinInt {
        return OdinInt {
            inner: u32::from_le_bytes(data),
        };
    }
}

impl fmt::Display for OdinInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl fmt::UpperHex for OdinInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:X}", self.inner)
    }
}

impl From<u32> for OdinInt {
    fn from(u: u32) -> Self {
        return OdinInt { inner: u };
    }
}

impl Into<u32> for OdinInt {
    fn into(self) -> u32 {
        return self.inner;
    }
}

impl From<bool> for OdinInt {
    fn from(b: bool) -> Self {
        return OdinInt {
            inner: if b { 1 } else { 0 },
        };
    }
}

impl From<OdinCmd> for OdinInt {
    fn from(cmd: OdinCmd) -> Self {
        match cmd {
            OdinCmd::ChunkTransferOk => OdinInt::from(0x00),
            OdinCmd::SessionStart => OdinInt::from(0x64),
            OdinCmd::TransferPIT => OdinInt::from(0x65),
            OdinCmd::Flash => OdinInt::from(0x66),
            OdinCmd::SessionEnd => OdinInt::from(0x67),
        }
    }
}
