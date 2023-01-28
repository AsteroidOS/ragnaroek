use crate::comms::Communicator;
use crate::Result;

use super::DownloadProtocolError;

const PING: [u8; 4] = [b'O', b'D', b'I', b'N'];
const PONG: [u8; 4] = [b'L', b'O', b'K', b'E'];

/// This should be invoked on the `Communicator` before any other command.
pub(crate) fn magic_handshake(c: &mut Box<dyn Communicator>) -> Result<()> {
    log::debug!(target: "DL", "Handshaking");
    c.send(&PING)?;
    // Some Samsung devices (Gear Watch 4, maybe more) require an empty bulk transfer to be sent after for handshake to continue.
    // c.send(&PING)?;
    let resp = c.recv_exact(PONG.len())?;
    if resp != PONG {
        return Err(DownloadProtocolError::InvalidMagicHandshake(resp).into());
    }

    log::debug!(target: "DL", "Handshake OK");

    return Ok(());
}
