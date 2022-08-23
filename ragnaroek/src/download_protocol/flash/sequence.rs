use super::super::*;
use crate::download_protocol::begin_session::SessionParams;
use crate::Communicator;
use crate::Result;
use pit::*;

// These values are correct for flashing without compression.
const FLASH_CMD_SEQUENCE_BEGIN: u32 = 0x02;
const FLASH_CMD_SEQUENCE_END: u32 = 0x03;

/// Tell the target to expect a file sequence (series of packets making up part of the file).
pub fn initiate(c: &mut Box<dyn Communicator>, sp: SessionParams) -> Result<()> {
    // Tell target that we want to flash
    let p = OdinCmdPacket::with_2_args(
        OdinCmd::Flash,
        OdinInt::from(FLASH_CMD_SEQUENCE_BEGIN),
        OdinInt::from(sp.max_seq_size_bytes),
    );
    p.send(c)?;

    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::Flash {
        return Err(DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::Flash, resp.cmd).into());
    }

    // For USB, an empty bulk transfer is expected before the first packet
    c.send(&[])?;

    return Ok(());
}

/// Send a packet of the file sequence to the target.
///
/// `data` should be no larger than the maximum negotiated packet size. However, that is not checked by this
/// function to allow for more flexible (ab)use.
fn send_packet(c: &mut Box<dyn Communicator>, packet: &[u8], packet_idx: OdinInt) -> Result<()> {
    log::debug!(target: "FLASH", "[Packet {}] Transferring", packet_idx);
    c.send(packet)?;

    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::Flash {
        return Err(DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::Flash, resp.cmd).into());
    }

    if resp.arg != packet_idx {
        return Err(DownloadProtocolError::UnexpectedFlashPacket(packet_idx, resp.arg).into());
    }

    log::debug!(target: "FLASH", "[Packet {}] OK", packet_idx);

    return Ok(());
}

/// Send an entire sequence of packets to the target.
///
/// This should be called once per sequence, after `initiate`.
pub fn transfer(
    c: &mut Box<dyn Communicator>,
    max_packet_size: usize,
    sequence: &[u8],
) -> Result<()> {
    for (packet_idx, packet) in sequence.chunks(max_packet_size).enumerate() {
        let packet_idx: u32 = packet_idx.try_into().unwrap();
        send_packet(c, packet, OdinInt::from(packet_idx))?;
    }
    return Ok(());
}

/// Tell the target to finish the sequence and write it to the given partition.
///
/// This should only be called after `initiate` and `transfer`.
pub fn end(
    c: &mut Box<dyn Communicator>,
    pit_entry: &PitEntry,
    sequence_length_bytes: OdinInt,
    is_last_sequence: bool,
) -> Result<()> {
    // For USB, an empty bulk transfer is expected before end
    c.send(&[])?;

    // AP and modem packets are the same, except for the added partition ID field for AP
    let is_modem: bool = pit_entry.pit_type == PitType::Modem;
    let p: OdinCmdPacket;
    let device_type: u32 = pit_entry.pit_device_type.into();
    if is_modem {
        p = OdinCmdPacket::with_6_args(
            OdinCmd::Flash,
            OdinInt::from(FLASH_CMD_SEQUENCE_END),
            OdinInt::from(is_modem),
            sequence_length_bytes,
            OdinInt::from(0x00),
            OdinInt::from(device_type),
            OdinInt::from(is_last_sequence),
        );
    } else {
        p = OdinCmdPacket::with_7_args(
            OdinCmd::Flash,
            OdinInt::from(FLASH_CMD_SEQUENCE_END),
            OdinInt::from(is_modem),
            sequence_length_bytes,
            OdinInt::from(0x00),
            OdinInt::from(device_type),
            OdinInt::from(pit_entry.pit_id),
            OdinInt::from(is_last_sequence),
        );
    }
    p.send(c)?;

    // For USB, an empty bulk transfer is expected after end
    c.send(&[])?;

    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::Flash {
        return Err(DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::Flash, resp.cmd).into());
    }

    return Ok(());
}
