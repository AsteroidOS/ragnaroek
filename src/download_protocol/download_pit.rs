use super::*;

use crate::pit::*;
use crate::Communicator;
use crate::Result;

const PIT_CHUNK_SIZE: usize = 500;

const PIT_FLAG_DUMP: u32 = 0x01;
const PIT_FLAG_CHUNK: u32 = 0x02;
const PIT_FLAG_END: u32 = 0x03;

const PIT_END_OK: u32 = 0x00;

/// Downloads partitioning data from the target.
/// Must be called after a session has been established.
/// TODO: Enforce this constraint via type system
pub fn download_pit(c: &mut Box<dyn Communicator>) -> Result<Pit> {
    log::info!(target: "PIT DL", "Start PIT download");
    let total_len: OdinInt = initiate_pit_download(c)?;
    let total_len: u32 = total_len.into();
    let total_len: usize = total_len
        .try_into()
        .expect("Not trying to run this on a 16-bit platform, are you?");
    let mut data: Vec<u8> = Vec::new();
    data.reserve(total_len);

    let mut chunk_idx: usize = 0;
    while data.len() < total_len {
        data.extend_from_slice(&fetch_pit_chunk(c, total_len - data.len(), chunk_idx)?);
        chunk_idx += 1;
    }

    end_pit_download(c)?;
    log::info!(target: "PIT DL", "PIT download OK");

    return Ok(Pit::deserialize(&data)?);
}

/// Sends the initial PIT download request packet and checks for an appropriate target response.
/// Returns either an Error or the amount of bytes the target is about to transfer.
/// The effects of calling this while a transfer is already in progress are unknown.
fn initiate_pit_download(c: &mut Box<dyn Communicator>) -> Result<OdinInt> {
    log::debug!(target: "PIT DL", "Initiating PIT download");
    let p = OdinCmdPacket::with_1_arg(OdinCmd::TransferPIT, OdinInt::from(PIT_FLAG_DUMP));
    p.send(c)?;

    // We expect an 8-byte response from the target
    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::TransferPIT {
        return Err(
            DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::TransferPIT, resp.cmd).into(),
        );
    }
    log::debug!(target: "PIT DL", "Initiating PIT download OK, {} bytes large", resp.arg);
    return Ok(resp.arg);
}

/// Puts in a request for the next chunk of PIT data with the target and fetches it.
fn fetch_pit_chunk(
    c: &mut Box<dyn Communicator>,
    total_remaining: usize,
    chunk_idx: usize,
) -> Result<Vec<u8>> {
    // Calculate which chunk index to use
    let chunk_idx: u32 = chunk_idx.try_into().unwrap();
    let chunk_idx: OdinInt = chunk_idx.into();
    log::debug!(target: "PIT DL", "[Chunk {}] Fetching", chunk_idx);

    // Send request
    let p = OdinCmdPacket::with_2_args(
        OdinCmd::TransferPIT,
        OdinInt::from(PIT_FLAG_CHUNK),
        chunk_idx,
    );
    p.send(c)?;

    // Read response
    let left = core::cmp::min(total_remaining, PIT_CHUNK_SIZE);
    let ret = c.recv_exact(left).map_err(|e| e.into());
    log::debug!(target: "PIT DL", "[Chunk {}] Fetching OK", chunk_idx);
    return ret;
}

/// Tells the target that the PIT transfer is over and checks for an appropriate target response.
/// The effects of calling this without initiating a transfer or in the middle of one are unknown.
fn end_pit_download(c: &mut Box<dyn Communicator>) -> Result<()> {
    log::debug!(target: "PIT DL", "Ending PIT download");

    // USB expects an empty bulk transfer after the last data chunk
    c.send(&[])?;

    let p = OdinCmdPacket::with_1_arg(OdinCmd::TransferPIT, OdinInt::from(PIT_FLAG_END));
    p.send(c)?;

    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::TransferPIT {
        return Err(
            DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::TransferPIT, resp.cmd).into(),
        );
    }
    if resp.arg != OdinInt::from(PIT_END_OK) {
        return Err(DownloadProtocolError::UnexpectedOdinCmdArg(
            OdinInt::from(PIT_END_OK),
            resp.arg,
        )
        .into());
    }
    log::debug!(target: "PIT DL", "Ending PIT download OK");

    return Ok(());
}
