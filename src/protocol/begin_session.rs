use super::*;

use crate::comms::{Communicator, Result};

const BEGIN_SESSION: u32 = 0x00;

/// Begins a session with a target.
pub fn begin_session(c: &mut Box<dyn Communicator>) -> Result<()> {
    // The intial command is always just BeginSession
    let p = OdinCmdPacket {
        kind: OdinCmd::SessionStart,
        arg1: OdinInt::from(BEGIN_SESSION),
        arg2: None,
    };

    p.send(c)?;

    // We expect an 8-byte response from the target
    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::SessionStart {
        panic!(
            "Target sent unexpected Odin command in reply: {:?}",
            resp.cmd
        );
    }
    println!("Reply: {:?}", resp);

    // TODO: The second command has strange fields set in the Samsung implementation. Do we need to send a second command?

    return Ok(());
}
