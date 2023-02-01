use super::*;
use crate::comms::Communicator;
use crate::Result;

/// Target mode to reboot into. OS is supported on all devices, the others might not be.
#[derive(Debug, Clone, Copy)]
pub enum ActionAfter {
    /// Do nothing. Device might not respond to commands after this.
    Nothing = 0x00,
    /// Reboot into the operating system. May be unsupported for older bootloaders.
    RebootOS = 0x01,
    /// Reboot into download mode. May be unsupported for older bootloaders.
    RebootOdin = 0x02,
    /// Shut down the target. May be unsupported for older bootloaders.
    Shutdown = 0x03,
}

/// Ends the targets session, with an optional reboot to the OS.
pub fn end_session(c: &mut Box<dyn Communicator>, after: ActionAfter) -> Result<()> {
    log::debug!(target: "SESS", "Ending session with action {:?}", after);
    // Heimdall always first sends a session end, and only then a reboot.
    // Not sure if needed or we could send a reboot request immediately.
    let p = OdinCmdPacket::with_1_arg(OdinCmd::SessionEnd, OdinInt::from(after as u32));
    p.send(c)?;

    // We expect an 8-byte response from the target
    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::SessionEnd {
        return Err(DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::SessionEnd, resp.cmd).into());
    }
    log::debug!(target: "SESS", "Ending session OK");
    return Ok(());
}
