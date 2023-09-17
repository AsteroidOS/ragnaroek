/// Error type returned when PIT file (de)serialization fails.
#[derive(Debug, Clone, PartialEq)]
pub enum PitError {
    /// PIT magic bytes were not entirely present.
    MagicTooShort,
    /// PIT magic bytes were not valid.
    InvalidPit([u8; 4]),
    /// PIT didn't contain enough data for a fixed-size field
    FieldTooShort(usize, usize),
    /// PIT contained a string that's not valid UTF-8
    InvalidUTF8(Vec<u8>),
    /// PIT partition binary type was not valid.
    InvalidBinaryType(u32),
    /// PIT partition storage device type was not valid.
    InvalidDeviceType(u32),
    /// PIT didn't contain any block data after the header.
    NoBlockData,
}
