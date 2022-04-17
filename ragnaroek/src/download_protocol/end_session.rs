use super::*;
use crate::comms::Communicator;
use crate::Result;

const END_SESSION: u32 = 0x00;
const REBOOT: u32 = 0x01;

/// Ends the targets session, with an optional reboot to the OS.
pub fn end_session(c: &mut Box<dyn Communicator>, reboot: bool) -> Result<()> {
    // Heimdall always first sends a session end, and only then a reboot.
    // Not sure if needed or we could send a reboot request immediately.
    let p = OdinCmdPacket::with_1_arg(OdinCmd::SessionEnd, OdinInt::from(END_SESSION));
    p.send(c)?;

    // We expect an 8-byte response from the target
    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::SessionEnd {
        return Err(DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::SessionEnd, resp.cmd).into());
    }

    if reboot {
        // Send a reboot
        let p = OdinCmdPacket::with_1_arg(OdinCmd::SessionEnd, OdinInt::from(REBOOT));
        p.send(c)?;

        let resp = OdinCmdReply::read(c)?;
        if resp.cmd != OdinCmd::SessionEnd {
            return Err(
                DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::SessionEnd, resp.cmd).into(),
            );
        }
    }

    return Ok(());
}
