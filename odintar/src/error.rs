use std::io;
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
