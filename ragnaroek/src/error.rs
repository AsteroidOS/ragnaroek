use crate::download_protocol::DownloadProtocolError;
use crate::upload_protocol::UploadProtocolError;

use core::result;
use std::io;

use pit::PitError;

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
    /// Transfer error was caused by an Odin protocol violation.
    DownloadProtocolError(DownloadProtocolError),
    /// Transfer error was caused by an upload mode protocol violation.
    UploadProtocolError(UploadProtocolError),
}

impl From<io::Error> for TransferError {
    fn from(e: io::Error) -> Self {
        return TransferError::IoError(e);
    }
}

impl From<DownloadProtocolError> for TransferError {
    fn from(e: DownloadProtocolError) -> Self {
        return TransferError::DownloadProtocolError(e);
    }
}

impl From<UploadProtocolError> for TransferError {
    fn from(e: UploadProtocolError) -> Self {
        return TransferError::UploadProtocolError(e);
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        let e = TransferError::IoError(e);
        return Error::TransferError(e);
    }
}

impl From<DownloadProtocolError> for Error {
    fn from(e: DownloadProtocolError) -> Self {
        let e = TransferError::DownloadProtocolError(e);
        return Error::TransferError(e);
    }
}

impl From<UploadProtocolError> for Error {
    fn from(e: UploadProtocolError) -> Self {
        let e = TransferError::UploadProtocolError(e);
        return Error::TransferError(e);
    }
}

impl From<PitError> for Error {
    fn from(e: PitError) -> Self {
        return Error::PitError(e);
    }
}
