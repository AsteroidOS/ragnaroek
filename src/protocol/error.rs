use super::{OdinCmd, OdinInt};

/// Error type returned when the protocol is violated.
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolError {
    /// An OdinInt was read which was expected to be a valid OdinCmd identifier, but wasn't
    InvalidOdinCmd(OdinInt),
    /// Target sent an unexpected OdinCmd identifier in a reply
    InvalidTargetReplyOdinCmd(OdinCmd, OdinCmd),
    /// Target sent an unexpected reply to the magic handshake
    InvalidMagicHandshake(Vec<u8>),
}
