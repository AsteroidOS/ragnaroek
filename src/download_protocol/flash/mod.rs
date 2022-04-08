mod sequence;

use super::*;

use crate::pit::PitEntry;
use crate::Communicator;
use crate::Result;

const FLASH_CMD_BEGIN_FLASH: u32 = 0x00;

/// The top-level flash function.
///
/// It chops the file up into flash sequences and sends them all to the target.
pub fn flash(c: &mut Box<dyn Communicator>, data: &[u8], pit_entry: PitEntry) -> Result<()> {
    let p = OdinCmdPacket::with_1_arg(OdinCmd::Flash, OdinInt::from(FLASH_CMD_BEGIN_FLASH));
    p.send(c)?;

    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::Flash {
        return Err(DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::Flash, resp.cmd).into());
    }

    let mut bytes_flashed: usize = 0;
    for sequence in data.chunks(sequence::MAX_SEQUENCE_SIZE_BYTES) {
        let sequence_len: u32 = sequence
            .len()
            .try_into()
            .expect("Sequence length too large to fit into u32! This is probably a bug");
        sequence::initiate(c, sequence_len)?;

        sequence::transfer(c, data)?;

        bytes_flashed += sequence.len();
        let is_last_sequence = bytes_flashed >= data.len();
        sequence::end(c, &pit_entry, OdinInt::from(sequence_len), is_last_sequence)?;
    }

    return Ok(());
}
