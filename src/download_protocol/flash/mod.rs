mod sequence;

use super::*;

use crate::Communicator;
use crate::Result;
use pit::PitEntry;

const FLASH_CMD_BEGIN_FLASH: u32 = 0x00;

/// The top-level flash function.
///
/// It chops the file up into flash sequences and sends them all to the target.
pub fn flash(c: &mut Box<dyn Communicator>, data: &[u8], pit_entry: PitEntry) -> Result<()> {
    log::info!(target: "FLASH", "Starting flash");
    let p = OdinCmdPacket::with_1_arg(OdinCmd::Flash, OdinInt::from(FLASH_CMD_BEGIN_FLASH));
    p.send(c)?;

    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::Flash {
        return Err(DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::Flash, resp.cmd).into());
    }
    log::info!(target: "FLASH", "Starting flash OK");

    log::debug!(target: "FLASH", "Starting flash file sequence transfers");
    let mut bytes_flashed: usize = 0;
    for (i, sequence) in data.chunks(sequence::MAX_SEQUENCE_SIZE_BYTES).enumerate() {
        let sequence_len: u32 = sequence
            .len()
            .try_into()
            .expect("Sequence length too large to fit into u32! This is probably a bug");
        log::debug!(target: "FLASH", "[Sequence {}] Starting transfer", i);
        sequence::initiate(c, sequence_len)?;
        log::debug!(target: "FLASH", "[Sequence {}] OK", i);

        log::debug!(target: "FLASH", "[Sequence {}] Transfering data", i);
        sequence::transfer(c, data)?;
        log::debug!(target: "FLASH", "[Sequence {}] OK", i);

        bytes_flashed += sequence.len();
        let is_last_sequence = bytes_flashed >= data.len();
        log::debug!(target: "FLASH", "[Sequence {}] Ending transfer", i);
        sequence::end(c, &pit_entry, OdinInt::from(sequence_len), is_last_sequence)?;
        log::debug!(target: "FLASH", "[Sequence {}] OK", i);
    }
    log::info!(target: "FLASH", "Flash OK");

    return Ok(());
}
