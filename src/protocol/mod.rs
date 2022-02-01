//! This module is the core of the actual protocol implementation.

mod begin_session;
mod download_pit;
mod end_session;
mod error;
mod magic_handshake;

pub use begin_session::begin_session;
pub use download_pit::download_pit;
pub use end_session::end_session;
pub use error::ProtocolError;
pub use magic_handshake::magic_handshake;

use crate::comms::Result;
use crate::Communicator;

/// Seems like all Odin command packets are exactly 1024 bytes long
const CMD_PACKET_LEN: usize = 1024;

/// The integral type used in the Odin protocol and the PIT format.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct OdinInt {
    inner: u32,
}

impl OdinInt {
    /// Convert to the wire representation.
    pub fn to_wire(&self) -> [u8; 4] {
        return u32::to_le_bytes(self.inner);
    }

    /// Construct from the wire representation.
    pub fn from_wire(data: [u8; 4]) -> OdinInt {
        return OdinInt {
            inner: u32::from_le_bytes(data),
        };
    }
}

impl From<u32> for OdinInt {
    fn from(u: u32) -> Self {
        return OdinInt { inner: u };
    }
}

impl Into<u32> for OdinInt {
    fn into(self) -> u32 {
        return self.inner;
    }
}

impl From<OdinCmd> for OdinInt {
    fn from(cmd: OdinCmd) -> Self {
        match cmd {
            OdinCmd::SessionStart => OdinInt::from(0x64),
            OdinCmd::TransferPIT => OdinInt::from(0x65),
            OdinCmd::Flash => OdinInt::from(0x66),
            OdinCmd::SessionEnd => OdinInt::from(0x67),
        }
    }
}

/// All known command IDs that can be sent to the target as the first Integer in a packet.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OdinCmd {
    SessionStart,
    TransferPIT,
    Flash,
    SessionEnd,
}

impl TryFrom<OdinInt> for OdinCmd {
    type Error = ProtocolError;
    fn try_from(int: OdinInt) -> std::result::Result<Self, Self::Error> {
        match int {
            OdinInt { inner: 0x64 } => Ok(OdinCmd::SessionStart),
            OdinInt { inner: 0x65 } => Ok(OdinCmd::TransferPIT),
            OdinInt { inner: 0x66 } => Ok(OdinCmd::Flash),
            OdinInt { inner: 0x67 } => Ok(OdinCmd::SessionEnd),
            _ => Err(ProtocolError::InvalidOdinCmd(int)),
        }
    }
}

/// Structure of all command packets.
/// These are always sent flasher -> target.
#[derive(Debug, Clone, PartialEq)]
struct OdinCmdPacket {
    kind: OdinCmd,
    arg1: OdinInt,
    arg2: Option<OdinInt>,
}

impl OdinCmdPacket {
    /// Send the constructed packet in the proper format over the given `Communicator`.
    fn send(&self, comm: &mut Box<dyn Communicator>) -> Result<()> {
        let mut buf: Vec<u8> = Vec::new();
        buf.reserve(CMD_PACKET_LEN);

        let cmd_int: OdinInt = self.kind.into();
        buf.extend_from_slice(&cmd_int.to_wire());
        buf.extend_from_slice(&self.arg1.to_wire());

        if self.arg2.is_some() {
            buf.extend_from_slice(&self.arg2.unwrap().to_wire());
        }

        // Has to be padded to minimum packet size
        buf.resize(CMD_PACKET_LEN, 0x00);

        return comm.send(&buf);
    }
}

/// Structure of the target's 8-byte reply to some of the command packets.
#[derive(Debug, Clone, Copy)]
struct OdinCmdReply {
    cmd: OdinCmd,
    arg: OdinInt,
}

impl OdinCmdReply {
    /// Read the reply from the given `Communicator`.
    /// Blocks until the complete reply could be read.
    fn read(c: &mut Box<dyn Communicator>) -> Result<OdinCmdReply> {
        let buf = c.recv_exact(8)?;

        // TODO: DRY
        let mut cmd_buf: [u8; 4] = [0; 4];
        cmd_buf[0] = buf[0];
        cmd_buf[1] = buf[1];
        cmd_buf[2] = buf[2];
        cmd_buf[3] = buf[3];
        let cmd_int = OdinInt::from_wire(cmd_buf);
        let cmd: OdinCmd = cmd_int
            .try_into()
            .expect("Target returned unknown Odin command");

        // TODO: DRY
        let mut arg_buf: [u8; 4] = [0; 4];
        arg_buf[0] = buf[4];
        arg_buf[1] = buf[5];
        arg_buf[2] = buf[6];
        arg_buf[3] = buf[7];
        let arg = OdinInt::from_wire(arg_buf);

        return Ok(OdinCmdReply { cmd, arg });
    }
}
