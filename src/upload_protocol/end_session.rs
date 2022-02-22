use crate::Communicator;
use crate::Result;

const POSTAMBLE: &[u8] = &[b'P', b'o', b'S', b't', b'A', b'm', b'B', b'l', b'E', b'\0'];

/// End a session with the target.
/// Must be called before disconnecting from the device.
/// Must not be called before performing the handshake.
pub fn end_session(c: &mut Box<dyn Communicator>) -> Result<()> {
    super::send_packet(c, POSTAMBLE)?;
    return Ok(());
}
