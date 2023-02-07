use std::io;
use std::num::{ParseIntError, TryFromIntError};
use std::string::FromUtf8Error;

/// Error returned when encountering an issue with a given Odin tar archive.
#[derive(Debug)]
pub enum OdinTarError {
    /// An error encountered when reading the underlying tar archive.
    IoError(io::Error),
    /// An error encountered when reading Odin's metadata in the archive.
    MetadataError(),
    /// Checksum mismatch between Odin's metadata and the actual contents.
    ChecksumError(String, String),
    /// Invalid UTF-8 in the Odin metadata.
    EncodingError(FromUtf8Error),
    /// Failure during integer casting. This probably indicates a library bug.
    IntConversionError(TryFromIntError),
    /// Failure during integer parsing. This probably indicates a library bug.
    IntParseError(ParseIntError),
}

impl From<io::Error> for OdinTarError {
    fn from(value: io::Error) -> Self {
        return OdinTarError::IoError(value);
    }
}

impl From<FromUtf8Error> for OdinTarError {
    fn from(value: FromUtf8Error) -> Self {
        return OdinTarError::EncodingError(value);
    }
}

impl From<TryFromIntError> for OdinTarError {
    fn from(value: TryFromIntError) -> Self {
        return OdinTarError::IntConversionError(value);
    }
}

impl From<ParseIntError> for OdinTarError {
    fn from(value: ParseIntError) -> Self {
        return OdinTarError::IntParseError(value);
    }
}
