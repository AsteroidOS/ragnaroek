use crate::pit::PitError;
use crate::protocol::ProtocolError;

use core::result;
use std::io;

/// Raganroek's top-level error type.
///
/// This is the only error type that public functions of this crate will ever return.
#[derive(Debug)]
pub enum Error {
    /// Error encountered while processing a PIT file.
    PitError(PitError),
    /// Error encountered while talking to the target.
    TransferError(TransferError),
}

/// Ragnaroek's top-level result type.
///
/// This is the only result type that public functions of this crate will ever return.
pub type Result<T> = result::Result<T, Error>;

/// Error type returned when an Odin protocol transfer fails.
#[derive(Debug)]
pub enum TransferError {
    /// Transfer error was caused by an I/O issue.
    IoError(io::Error),
    /// Transfer error was caused by a protocol violation.
    ProtocolError(ProtocolError),
}

impl From<io::Error> for TransferError {
    fn from(e: io::Error) -> Self {
        return TransferError::IoError(e);
    }
}

impl From<ProtocolError> for TransferError {
    fn from(e: ProtocolError) -> Self {
        return TransferError::ProtocolError(e);
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        let e = TransferError::IoError(e);
        return Error::TransferError(e);
    }
}

impl From<ProtocolError> for Error {
    fn from(e: ProtocolError) -> Self {
        let e = TransferError::ProtocolError(e);
        return Error::TransferError(e);
    }
}

impl From<PitError> for Error {
    fn from(e: PitError) -> Self {
        return Error::PitError(e);
    }
}
