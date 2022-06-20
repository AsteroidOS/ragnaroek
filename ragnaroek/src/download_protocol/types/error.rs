use super::{OdinCmd, OdinInt};

/// Error type returned when the protocol is violated.
#[derive(Debug, Clone, PartialEq)]
pub enum DownloadProtocolError {
    /// Target speaks an unknown protocol version.
    UnknownProtoVersion(OdinInt),
    /// Target sent an invalid OdinCmd identifier in a reply.
    ///
    /// The argument is this identifier in it's OdinInt form.
    InvalidOdinCmd(OdinInt),
    /// Target sent an invalid argument in a reply.
    ///
    /// The arguments are the expected argument and the actual argument.
    UnexpectedOdinCmdArg(OdinInt, OdinInt),
    /// Target sent an unexpected OdinCmd identifier in a reply.
    ///
    /// The arguments are the expected command and the actual command.
    UnexpectedOdinCmd(OdinCmd, OdinCmd),
    /// Target sent an unexpected reply to the magic handshake.
    ///
    /// The argument is this reply.
    InvalidMagicHandshake(Vec<u8>),
    /// Target reported a failure to flash a file part.
    ReportedPartFlashFailure,
    /// Target sent an unexpected file flash part number in reply.
    ///
    /// The arguments are the expected part number and the actual part number.
    UnexpectedFlashPart(OdinInt, OdinInt),
}
