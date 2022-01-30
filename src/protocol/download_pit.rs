use super::*;

use crate::pit::*;
use crate::Communicator;
use crate::Result;

const PIT_CHUNK_SIZE: usize = 500;

const PIT_FLAG_DUMP: u32 = 0x01;
const PIT_FLAG_CHUNK: u32 = 0x02;
const PIT_FLAG_END: u32 = 0x03;

/// Downloads partitioning data from the target.
/// Must be called after a session has been established.
/// TODO: Enforce this constraint via type system
pub fn download_pit(c: &mut Box<dyn Communicator>) -> Result<Pit> {
    let total_len: OdinInt = initiate_pit_download(c)?;
    let total_len: u32 = total_len.into();
    let total_len: usize = total_len
        .try_into()
        .expect("Not trying to run this on a 16-bit platform, are you?");
    let mut have_len: usize = 0;
    let mut data: Vec<u8> = Vec::new();
    data.reserve(total_len);

    while have_len < total_len {
        data.extend_from_slice(&fetch_pit_chunk(c, have_len, total_len)?);
        have_len = data.len();
    }

    end_pit_download(c)?;

    return Ok(Pit::deserialize(&data)?);
}

/// Sends the initial PIT download request packet and checks for an appropriate target response.
/// Returns either an Error or the amount of bytes the target is about to transfer.
/// The effects of calling this while a transfer is already in progress are unknown.
fn initiate_pit_download(c: &mut Box<dyn Communicator>) -> Result<OdinInt> {
    let p = OdinCmdPacket {
        kind: OdinCmd::TransferPIT,
        arg1: OdinInt::from(PIT_FLAG_DUMP),
        arg2: None,
    };
    p.send(c)?;

    // We expect an 8-byte response from the target
    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::TransferPIT {
        return Err(
            ProtocolError::InvalidTargetReplyOdinCmd(OdinCmd::TransferPIT, resp.cmd).into(),
        );
    }
    return Ok(resp.arg);
}

/// Puts in a request for the next chunk of PIT data with the target and fetches it.
/// Returns an error or the read data.
/// Only call as long as have_len < total_len (AKA the transfer isn't done yet).
fn fetch_pit_chunk(
    c: &mut Box<dyn Communicator>,
    have_len: usize,
    total_len: usize,
) -> Result<Vec<u8>> {
    // Calculate which chunk index to use
    let chunk_idx: usize = have_len / PIT_CHUNK_SIZE;
    let chunk_idx: u32 = chunk_idx.try_into().unwrap();
    let chunk_idx: OdinInt = chunk_idx.into();

    // Send request
    let p = OdinCmdPacket {
        kind: OdinCmd::TransferPIT,
        arg1: OdinInt::from(PIT_FLAG_CHUNK),
        arg2: Some(chunk_idx),
    };
    p.send(c)?;

    // Read response
    let left = core::cmp::min(total_len - have_len, PIT_CHUNK_SIZE);
    return c.recv_exact(left).map_err(|e| e.into());
}

/// Tells the target that the PIT transfer is over and checks for an appropriate target response.
/// The effects of calling this without initiating a transfer or in the middle of one are unknown.
fn end_pit_download(c: &mut Box<dyn Communicator>) -> Result<()> {
    let p = OdinCmdPacket {
        kind: OdinCmd::TransferPIT,
        arg1: OdinInt::from(PIT_FLAG_END),
        arg2: None,
    };
    p.send(c)?;

    // We expect an 8-byte response from the target
    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::TransferPIT {
        return Err(
            ProtocolError::InvalidTargetReplyOdinCmd(OdinCmd::TransferPIT, resp.cmd).into(),
        );
    }

    return Ok(());
}
