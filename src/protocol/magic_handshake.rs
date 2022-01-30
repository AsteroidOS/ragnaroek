use crate::comms::Communicator;
use crate::Result;

use super::ProtocolError;

const PING: [u8; 4] = [b'O', b'D', b'I', b'N'];
const PONG: [u8; 4] = [b'L', b'O', b'K', b'E'];

/// This should be invoked on the `Communicator` before any other command.
pub fn magic_handshake(c: &mut Box<dyn Communicator>) -> Result<()> {
    c.send(&PING)?;
    let resp = c.recv_exact(PONG.len())?;
    if resp != PONG {
        return Err(ProtocolError::InvalidMagicHandshake(resp).into());
    }

    return Ok(());
}
