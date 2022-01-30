use crate::protocol::OdinInt;

/// Error type returned when PIT file (de)serialization fails.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PitError {
    /// PIT magic bytes were not valid.
    InvalidPit([u8; 4]),
    /// PIT partition binary type was not valid.
    InvalidBinaryType(OdinInt),
    /// PIT partition storage device type was not valid.
    InvalidDeviceType(OdinInt),
    /// PIT string was too long.
    PitStringTooLong(usize),
}
