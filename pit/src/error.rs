/// Error type returned when PIT file (de)serialization fails.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PitError {
    /// PIT magic bytes were not valid.
    InvalidPit([u8; 4]),
    /// PIT partition binary type was not valid.
    InvalidBinaryType(u32),
    /// PIT partition storage device type was not valid.
    InvalidDeviceType(u32),
}
