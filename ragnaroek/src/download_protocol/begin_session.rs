use super::*;
use crate::Result;

use crate::comms::Communicator;

const BEGIN_SESSION: u32 = 0x00;
const SET_TOTAL_SIZE: u32 = 0x02;
const SET_PART_SIZE: u32 = 0x05;

// TODO: Query the device to determine max supported version
// const PROTO_VERSION: u32 = 0x04;
const PROTO_VERSION: u32 = 0x03;

/// Begins a session with a target.
pub fn begin_session(c: &mut Box<dyn Communicator>) -> Result<()> {
    // The intial command is always just BeginSession
    let p = OdinCmdPacket::with_2_args(
        OdinCmd::SessionStart,
        OdinInt::from(BEGIN_SESSION),
        OdinInt::from(PROTO_VERSION),
    );
    p.send(c)?;

    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::SessionStart {
        return Err(
            DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::SessionStart, resp.cmd).into(),
        );
    }
    // TODO: Check whether proto version matches
    let p = OdinCmdPacket::with_2_args(
        OdinCmd::SessionStart,
        OdinInt::from(SET_PART_SIZE),
        OdinInt::from(131072),
    );
    p.send(c)?;

    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::SessionStart {
        return Err(
            DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::SessionStart, resp.cmd).into(),
        );
    };
    return Ok(());

    // Set the total length. Not sure if this refers to the file or what.
    // FIXME: This is only safe to do if device doesn't return version 0
    let p = OdinCmdPacket::with_2_args(
        OdinCmd::SessionStart,
        OdinInt::from(SET_TOTAL_SIZE),
        OdinInt::from(0x4a7fa500),
    );
    p.send(c)?;

    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::SessionStart {
        return Err(
            DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::SessionStart, resp.cmd).into(),
        );
    }

    return Ok(());
}
