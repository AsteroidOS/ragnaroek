use super::*;

use crate::{Communicator, Result};

use core::fmt;

/// Seems like all Odin command packets are exactly 1024 bytes long
const CMD_PACKET_LEN: usize = 1024;

/// Structure of all command packets.
/// These are always sent flasher -> target.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct OdinCmdPacket {
    cmd: OdinCmd,
    arg1: OdinInt,
    arg2: Option<OdinInt>,
    arg3: Option<OdinInt>,
    arg4: Option<OdinInt>,
    arg5: Option<OdinInt>,
    arg6: Option<OdinInt>,
    arg7: Option<OdinInt>,
}

impl OdinCmdPacket {
    /// Construct a packet with a single argument.
    pub fn with_1_arg(cmd: OdinCmd, arg1: OdinInt) -> OdinCmdPacket {
        return OdinCmdPacket {
            cmd,
            arg1,
            arg2: None,
            arg3: None,
            arg4: None,
            arg5: None,
            arg6: None,
            arg7: None,
        };
    }

    pub fn with_2_args(kind: OdinCmd, arg1: OdinInt, arg2: OdinInt) -> OdinCmdPacket {
        let mut p = OdinCmdPacket::with_1_arg(kind, arg1);
        p.arg2 = Some(arg2);
        return p;
    }

    pub fn with_3_args(
        kind: OdinCmd,
        arg1: OdinInt,
        arg2: OdinInt,
        arg3: OdinInt,
    ) -> OdinCmdPacket {
        let mut p = OdinCmdPacket::with_2_args(kind, arg1, arg2);
        p.arg3 = Some(arg3);
        return p;
    }

    pub fn with_4_args(
        kind: OdinCmd,
        arg1: OdinInt,
        arg2: OdinInt,
        arg3: OdinInt,
        arg4: OdinInt,
    ) -> OdinCmdPacket {
        let mut p = OdinCmdPacket::with_3_args(kind, arg1, arg2, arg3);
        p.arg4 = Some(arg4);
        return p;
    }

    pub fn with_5_args(
        kind: OdinCmd,
        arg1: OdinInt,
        arg2: OdinInt,
        arg3: OdinInt,
        arg4: OdinInt,
        arg5: OdinInt,
    ) -> OdinCmdPacket {
        let mut p = OdinCmdPacket::with_4_args(kind, arg1, arg2, arg3, arg4);
        p.arg5 = Some(arg5);
        return p;
    }

    pub fn with_6_args(
        kind: OdinCmd,
        arg1: OdinInt,
        arg2: OdinInt,
        arg3: OdinInt,
        arg4: OdinInt,
        arg5: OdinInt,
        arg6: OdinInt,
    ) -> OdinCmdPacket {
        let mut p = OdinCmdPacket::with_5_args(kind, arg1, arg2, arg3, arg4, arg5);
        p.arg6 = Some(arg6);
        return p;
    }

    pub fn with_7_args(
        kind: OdinCmd,
        arg1: OdinInt,
        arg2: OdinInt,
        arg3: OdinInt,
        arg4: OdinInt,
        arg5: OdinInt,
        arg6: OdinInt,
        arg7: OdinInt,
    ) -> OdinCmdPacket {
        let mut p = OdinCmdPacket::with_6_args(kind, arg1, arg2, arg3, arg4, arg5, arg6);
        p.arg7 = Some(arg7);
        return p;
    }

    /// Send the constructed packet in the proper format over the given `Communicator`.
    pub(crate) fn send(&self, comm: &mut Box<dyn Communicator>) -> Result<()> {
        let mut buf: Vec<u8> = Vec::new();
        buf.reserve(CMD_PACKET_LEN);

        let cmd_int: OdinInt = self.cmd.into();
        buf.extend_from_slice(&cmd_int.to_wire());
        buf.extend_from_slice(&self.arg1.to_wire());

        if self.arg2.is_some() {
            buf.extend_from_slice(&self.arg2.unwrap().to_wire());
        }
        if self.arg3.is_some() {
            buf.extend_from_slice(&self.arg3.unwrap().to_wire());
        }
        if self.arg4.is_some() {
            buf.extend_from_slice(&self.arg4.unwrap().to_wire());
        }
        if self.arg5.is_some() {
            buf.extend_from_slice(&self.arg5.unwrap().to_wire());
        }
        if self.arg6.is_some() {
            buf.extend_from_slice(&self.arg6.unwrap().to_wire());
        }
        if self.arg7.is_some() {
            buf.extend_from_slice(&self.arg7.unwrap().to_wire());
        }

        // Has to be padded to minimum packet size
        buf.resize(CMD_PACKET_LEN, 0x00);

        log::trace!(target: "CMD", "{}", self);
        match comm.send(&buf) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

impl fmt::Display for OdinCmdPacket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = format!("Cmd: {:?}, Arg1: 0x{:X}", self.cmd, self.arg1);
        if self.arg2.is_some() {
            s.push_str(&format!(", Arg2: 0x{:X}", self.arg2.unwrap()));
        }
        if self.arg3.is_some() {
            s.push_str(&format!(", Arg3: 0x{:X}", self.arg3.unwrap()));
        }
        if self.arg4.is_some() {
            s.push_str(&format!(", Arg4: 0x{:X}", self.arg4.unwrap()));
        }
        if self.arg5.is_some() {
            s.push_str(&format!(", Arg5: 0x{:X}", self.arg5.unwrap()));
        }
        if self.arg6.is_some() {
            s.push_str(&format!(", Arg6: 0x{:X}", self.arg6.unwrap()));
        }
        if self.arg7.is_some() {
            s.push_str(&format!(", Arg7: 0x{:X}", self.arg7.unwrap()));
        }

        write!(f, "{}", s)
    }
}
