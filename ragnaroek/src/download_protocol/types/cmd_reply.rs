use super::*;
use crate::{Communicator, Result};

use core::fmt;

/// Structure of the target's 8-byte reply to some of the command packets.
#[derive(Debug, Clone, Copy)]
pub(crate) struct OdinCmdReply {
    pub(crate) cmd: OdinCmd,
    pub(crate) arg: OdinInt,
}

impl OdinCmdReply {
    /// Read the reply from the given `Communicator`.
    /// Blocks until the complete reply could be read.
    pub(crate) fn read(c: &mut Box<dyn Communicator>) -> Result<OdinCmdReply> {
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

        let reply = OdinCmdReply { cmd, arg };
        log::trace!(target: "CMD", "{}", reply);
        return Ok(reply);
    }
}

impl fmt::Display for OdinCmdReply {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cmd: {:?}, Arg: 0x{:X}", self.cmd, self.arg)
    }
}
