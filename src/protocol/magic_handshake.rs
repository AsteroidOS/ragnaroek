use crate::comms::{Communicator, Result};

use std::io::{Error, ErrorKind};

const PING: [u8; 4] = [b'O', b'D', b'I', b'N'];
const PONG: [u8; 4] = [b'L', b'O', b'K', b'E'];

/// This should be invoked on the `Communicator` before any other command.
pub fn magic_handshake(c: &mut Box<dyn Communicator>) -> Result<()> {
    c.send(&PING)?;
    let resp = c.recv_exact(PONG.len())?;
    if resp != PONG {
        // TODO: Think about error types
        return Err(Error::new(
            ErrorKind::InvalidData,
            format!(
                "Session init failed due to invalid data from target ({:?})",
                resp
            ),
        ));
    }

    return Ok(());
}
