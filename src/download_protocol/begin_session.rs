use super::*;
use crate::Result;

use crate::comms::Communicator;

const BEGIN_SESSION: u32 = 0x00;
// Not sure whether all devices support this version.
const PROTO_VERSION: u32 = 0x04;

/// Begins a session with a target.
pub fn begin_session(c: &mut Box<dyn Communicator>) -> Result<()> {
    // The intial command is always just BeginSession
    let p = OdinCmdPacket::with_2_args(
        OdinCmd::SessionStart,
        OdinInt::from(BEGIN_SESSION),
        OdinInt::from(PROTO_VERSION),
    );
    p.send(c)?;

    // We expect an 8-byte response from the target
    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::SessionStart {
        return Err(
            DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::SessionStart, resp.cmd).into(),
        );
    }

    // TODO: The second command has strange fields set in the Samsung implementation. Do we need to send a second command?

    return Ok(());
}
