use super::*;

use crate::download_protocol::begin_session::{ProtoVersion, SessionParams};
use crate::Communicator;
use crate::Result;

const PIT_CHUNK_SIZE: usize = 500;

const PIT_FLAG_FLASH: u32 = 0x00;
const PIT_FLAG_DUMP: u32 = 0x01;
const PIT_FLAG_CHUNK: u32 = 0x02;
const PIT_FLAG_END: u32 = 0x03;

const PIT_END_OK: u32 = 0x00;

/// Uploads partitioning data to the target.
pub(crate) fn flash_pit(
    c: &mut Box<dyn Communicator>,
    params: SessionParams,
    pit: &[u8],
) -> Result<()> {
    log::info!(target: "PIT", "Start PIT flash");
    let total_len: u32 = pit.len().try_into()?;
    let total_len: OdinInt = total_len.into();
    initiate_pit_flash(c, total_len)?;
    send_pit_data(c, pit)?;
    let is_proto_v3plus: bool =
        params.proto_version == ProtoVersion::V3 || params.proto_version == ProtoVersion::V4;
    end_pit_flash(c, is_proto_v3plus)?;
    log::info!(target: "PIT", "PIT flash OK");
    return Ok(());
}

/// Tells the target that we want to start sending PIT data.
fn initiate_pit_flash(c: &mut Box<dyn Communicator>, size: OdinInt) -> Result<()> {
    log::debug!(target: "PIT", "Initiating PIT flash");
    let p = OdinCmdPacket::with_1_arg(OdinCmd::TransferPIT, OdinInt::from(PIT_FLAG_FLASH));
    p.send(c)?;

    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::TransferPIT {
        return Err(
            DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::TransferPIT, resp.cmd).into(),
        );
    }
    if resp.arg != OdinInt::from(0x00) {
        return Err(
            DownloadProtocolError::UnexpectedOdinCmdArg(OdinInt::from(0x00), resp.arg).into(),
        );
    }
    log::debug!(target: "PIT", "Initiating PIT flash OK");

    log::debug!(target: "PIT", "Sending PIT size to target");
    let p = OdinCmdPacket::with_2_args(OdinCmd::TransferPIT, OdinInt::from(PIT_FLAG_CHUNK), size);
    p.send(c)?;

    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::TransferPIT {
        return Err(
            DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::TransferPIT, resp.cmd).into(),
        );
    }
    if resp.arg != OdinInt::from(0x00) {
        return Err(
            DownloadProtocolError::UnexpectedOdinCmdArg(OdinInt::from(0x00), resp.arg).into(),
        );
    }
    log::debug!(target: "PIT", "Sending PIT size to target OK");

    return Ok(());
}

/// Puts in a request for the next chunk of PIT data with the target and fetches it.
fn send_pit_data(c: &mut Box<dyn Communicator>, pit: &[u8]) -> Result<()> {
    log::debug!(target: "PIT", "Sending PIT data to target");
    c.send(pit)?;

    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::TransferPIT {
        return Err(
            DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::TransferPIT, resp.cmd).into(),
        );
    }
    if resp.arg != OdinInt::from(0x03) {
        return Err(
            DownloadProtocolError::UnexpectedOdinCmdArg(OdinInt::from(0x03), resp.arg).into(),
        );
    }
    log::debug!(target: "PIT", "Sending PIT data to target OK");

    return Ok(());
}

/// Tells the target that the PIT transfer is over and checks for an appropriate target response.
fn end_pit_flash(c: &mut Box<dyn Communicator>, is_proto_v3plus: bool) -> Result<()> {
    log::debug!(target: "PIT", "Ending PIT flash");

    // For whatever reason, if connected via USB the device really wants to send us an empty transfer
    // NOTE: Some protocol versions require these empty transfers, whether it's version 3 exactly is a guess.
    if is_proto_v3plus {
        log::debug!(target: "PIT", "Protocol version >3, exchanging empty transfers");
        log::trace!(target: "PIT", "Receiving empty transfer");
        c.recv_exact(0)?;
        log::trace!(target: "PIT", "Receiving empty transfer OK");

        // And the device expects an empty transfer from us
        log::trace!(target: "PIT", "Sending empty transfer");
        c.send(&[])?;
        log::trace!(target: "PIT", "Sending empty transfer OK");
    } else {
        log::debug!(target: "PIT", "Protocol version < 3, not exchanging empty transfers");
    }

    let p = OdinCmdPacket::with_1_arg(OdinCmd::TransferPIT, OdinInt::from(PIT_FLAG_END));
    p.send(c)?;

    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::TransferPIT {
        return Err(
            DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::TransferPIT, resp.cmd).into(),
        );
    }
    if resp.arg != OdinInt::from(0x00) {
        return Err(
            DownloadProtocolError::UnexpectedOdinCmdArg(OdinInt::from(0x00), resp.arg).into(),
        );
    }
    log::debug!(target: "PIT", "Ending PIT flash OK");

    return Ok(());
}
