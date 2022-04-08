use super::{DownloadProtocolError, OdinInt};

/// All known command IDs that can be sent to the target as the first Integer in a packet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OdinCmd {
    /// Not a real Odin command, but something sent by the target
    /// during the flashing process in the location where a Odin command
    /// would usually go.
    /// Including it as a variant here makes the code simpler.
    ChunkTransferOk,
    SessionStart,
    TransferPIT,
    Flash,
    SessionEnd,
}

impl TryFrom<OdinInt> for OdinCmd {
    type Error = DownloadProtocolError;
    fn try_from(int: OdinInt) -> std::result::Result<Self, Self::Error> {
        match int {
            OdinInt { inner: 0x00 } => Ok(OdinCmd::ChunkTransferOk),
            OdinInt { inner: 0x64 } => Ok(OdinCmd::SessionStart),
            OdinInt { inner: 0x65 } => Ok(OdinCmd::TransferPIT),
            OdinInt { inner: 0x66 } => Ok(OdinCmd::Flash),
            OdinInt { inner: 0x67 } => Ok(OdinCmd::SessionEnd),
            _ => Err(DownloadProtocolError::InvalidOdinCmd(int)),
        }
    }
}
