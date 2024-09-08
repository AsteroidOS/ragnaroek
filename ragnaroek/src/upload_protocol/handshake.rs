use std::sync::Arc;

use crate::error::TransferError;
use crate::Communicator;
use crate::Error;
use crate::Result;

use super::UploadProtocolError;

const PREAMBLE: &[u8] = b"PrEaMbLe\0";
const ACKNOWLEDGMENT: &[u8] = b"AcKnOwLeDgMeNt\0";

/// Handshake with the target.
/// This must be called before performing any other upload mode operations.
pub fn handshake(c: &mut Box<dyn Communicator>) -> Result<()> {
    super::send_packet(c, PREAMBLE)?;

    match c.recv_exact(ACKNOWLEDGMENT.len()) {
        Err(e) => return Err(Error::TransferError(TransferError::Io(Arc::new(e)))),
        Ok(data) => {
            if data != ACKNOWLEDGMENT {
                return Err(Error::TransferError(TransferError::UploadProtocol(
                    UploadProtocolError::MissingAck,
                )));
            }
        }
    }

    return Ok(());
}
