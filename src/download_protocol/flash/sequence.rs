use super::super::{OdinCmd, OdinCmdPacket, OdinCmdReply, OdinInt};
use crate::pit::*;
use crate::Communicator;
use crate::Result;

/// The maximal amount of data that can be transfered in a single sequence with default settings.
///
/// Non-default settings are not yet supported.
pub const MAX_SEQUENCE_SIZE_BYTES: usize = PART_MAX_SIZE * PART_MAX_COUNT;

const FLASH_CMD_SEQUENCE_BEGIN: u32 = 0x02;
const FLASH_CMD_SEQUENCE_END: u32 = 0x03;

const PART_MAX_SIZE: usize = 131072; // 128KiB
const PART_MAX_COUNT: usize = 800;

const FLASH_FAILURE: u32 = 0xFF;

/// Tell the target to expect a file sequence (series of packets making up part of the file).
pub fn initiate(c: &mut Box<dyn Communicator>, sequence_size_bytes: u32) -> Result<()> {
    // Tell target that we want to flash
    let p = OdinCmdPacket::with_2_args(
        OdinCmd::Flash,
        OdinInt::from(FLASH_CMD_SEQUENCE_BEGIN),
        OdinInt::from(sequence_size_bytes),
    );
    p.send(c)?;

    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::Flash {
        panic!(
            "Target sent unexpected Odin command in reply: {:?}",
            resp.cmd
        );
    }

    // For USB, an empty bulk transfer is expected before the first part
    c.send(&[])?;

    return Ok(());
}

/// Send a part of the file sequence to the target.
///
/// `data` should be no larger than the maximum part size. However, that is not checked by this
/// function to allow for more flexible (ab)use.
fn transfer_part(c: &mut Box<dyn Communicator>, part: &[u8], part_idx: OdinInt) -> Result<()> {
    c.send(part)?;

    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::Flash {
        panic!(
            "Target sent unexpected Odin command in reply: {:?}",
            resp.cmd
        );
    }

    // TODO: Retry
    if resp.arg == OdinInt::from(FLASH_FAILURE) {
        panic!("Target reported failure to flash part of sequence");
    }

    return Ok(());
}

/// Send an entire sequence of parts to the target.
///
/// This should be called once per sequence, after `initiate`.
pub fn transfer(c: &mut Box<dyn Communicator>, data: &[u8]) -> Result<()> {
    for (part_idx, part) in data.chunks(PART_MAX_SIZE).enumerate() {
        let part_idx: u32 = part_idx.try_into().unwrap();
        transfer_part(c, part, OdinInt::from(part_idx))?;
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
    if is_modem {
        p = OdinCmdPacket::with_6_args(
            OdinCmd::Flash,
            OdinInt::from(FLASH_CMD_SEQUENCE_END),
            OdinInt::from(is_modem),
            sequence_length_bytes,
            OdinInt::from(0x00),
            pit_entry.pit_device_type.into(),
            OdinInt::from(is_last_sequence),
        );
    } else {
        p = OdinCmdPacket::with_7_args(
            OdinCmd::Flash,
            OdinInt::from(FLASH_CMD_SEQUENCE_END),
            OdinInt::from(is_modem),
            sequence_length_bytes,
            OdinInt::from(0x00),
            pit_entry.pit_device_type.into(),
            pit_entry.pit_id,
            OdinInt::from(is_last_sequence),
        );
    }
    p.send(c)?;

    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::Flash {
        panic!(
            "Target sent unexpected Odin command in reply: {:?}",
            resp.cmd
        );
    }

    // For USB, an empty bulk transfer is expected after end
    c.send(&[])?;

    return Ok(());
}
