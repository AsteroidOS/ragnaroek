use crate::download_protocol::DownloadProtocolError;
use crate::upload_protocol::UploadProtocolError;

use core::result;
use std::io;
use std::num::TryFromIntError;
use std::sync::Arc;

use odintar::OdinTarError;
use pit::PitError;

/// Raganroek's top-level error type.
///
/// This is the only error type that public functions of this crate will ever return.
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub enum TransferError {
    /// Transfer error was caused by an I/O issue.
    Io(Arc<io::Error>),
    /// Transfer error was caused by an Odin protocol violation.
    DownloadProtocol(DownloadProtocolError),
    /// Transfer error was caused by an upload mode protocol violation.
    UploadProtocol(UploadProtocolError),
    /// Transfer error was caused by a failing integer conversion. This is probably a bug in ragnaroek.
    IntegerConversion(TryFromIntError),
    /// Transfer error was caused by a corrupt Odin TAR file.
    OdinTar(OdinTarError),
}

impl From<io::Error> for TransferError {
    fn from(e: io::Error) -> Self {
        return TransferError::Io(Arc::new(e));
    }
}

impl From<DownloadProtocolError> for TransferError {
    fn from(e: DownloadProtocolError) -> Self {
        return TransferError::DownloadProtocol(e);
    }
}

impl From<UploadProtocolError> for TransferError {
    fn from(e: UploadProtocolError) -> Self {
        return TransferError::UploadProtocol(e);
    }
}

impl From<TryFromIntError> for TransferError {
    fn from(e: TryFromIntError) -> Self {
        return TransferError::IntegerConversion(e);
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        let e = TransferError::Io(Arc::new(e));
        return Error::TransferError(e);
    }
}

impl From<DownloadProtocolError> for Error {
    fn from(e: DownloadProtocolError) -> Self {
        let e = TransferError::DownloadProtocol(e);
        return Error::TransferError(e);
    }
}

impl From<UploadProtocolError> for Error {
    fn from(e: UploadProtocolError) -> Self {
        let e = TransferError::UploadProtocol(e);
        return Error::TransferError(e);
    }
}

impl From<PitError> for Error {
    fn from(e: PitError) -> Self {
        return Error::PitError(e);
    }
}

impl From<TransferError> for Error {
    fn from(e: TransferError) -> Self {
        return Error::TransferError(e);
    }
}

impl From<TryFromIntError> for Error {
    fn from(e: TryFromIntError) -> Self {
        return Error::from(TransferError::IntegerConversion(e));
    }
}
