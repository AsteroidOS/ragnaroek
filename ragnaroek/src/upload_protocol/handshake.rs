use crate::error::TransferError;
use crate::Communicator;
use crate::Error;
use crate::Result;

use super::UploadProtocolError;

const PREAMBLE: &[u8] = &[b'P', b'r', b'E', b'a', b'M', b'b', b'L', b'e', b'\0'];
const ACKNOWLEDGMENT: &[u8] = &[
    b'A', b'c', b'K', b'n', b'O', b'w', b'L', b'e', b'D', b'g', b'M', b'e', b'N', b't', b'\0',
];

/// Handshake with the target.
/// This must be called before performing any other upload mode operations.
pub fn handshake(c: &mut Box<dyn Communicator>) -> Result<()> {
    super::send_packet(c, PREAMBLE)?;

    match c.recv_exact(ACKNOWLEDGMENT.len()) {
        Err(e) => return Err(Error::TransferError(TransferError::IoError(e))),
        Ok(data) => {
            if data != ACKNOWLEDGMENT {
                return Err(Error::TransferError(TransferError::UploadProtocolError(
                    UploadProtocolError::MissingAck,
                )));
            }
        }
    }

    return Ok(());
}
