mod sequence;

use super::begin_session::SessionParams;
use super::*;

use crate::Communicator;
use crate::Result;

use either::Either;
use pit::{PitEntryV1, PitEntryV2};

const FLASH_CMD_BEGIN_FLASH: u32 = 0x00;
const SET_TOTAL_SIZE: u32 = 0x02;

/// The top-level flash function.
///
/// It chops the file up into flash sequences and sends them all to the target.
pub(crate) fn flash(
    c: &mut Box<dyn Communicator>,
    sp: SessionParams,
    data: &[u8],
    pit_entry: Either<PitEntryV1, PitEntryV2>,
) -> Result<()> {
    log::info!(target: "FLASH", "Starting flash of {} bytes total", data.len());
    set_total_size(c, data)?;
    start(c)?;

    let total_seqs: usize = div_up(data.len(), sp.max_seq_size_bytes as usize);
    log::debug!(target: "FLASH", "Starting flash file sequence transfers, total sequences: {}", total_seqs);
    let mut bytes_flashed: usize = 0;
    for (i, sequence) in data.chunks(sp.max_seq_size_bytes as usize).enumerate() {
        let sequence_len: u32 = sequence
            .len()
            .try_into()
            .expect("Sequence length too large to fit into u32! This is probably a bug");
        log::debug!(target: "FLASH", "[Sequence {}/{}] Starting transfer of {} bytes", i + 1, total_seqs, sequence_len);
        sequence::initiate(c, sequence_len)?;
        log::debug!(target: "FLASH", "[Sequence {}/{}] OK", i + 1, total_seqs);

        log::debug!(target: "FLASH", "[Sequence {}/{}] Transferring data", i + 1, total_seqs);
        sequence::transfer(c, sp.max_packet_size as usize, data)?;
        log::debug!(target: "FLASH", "[Sequence {}/{}] OK", i + 1, total_seqs);

        bytes_flashed += sequence.len();
        let is_last_sequence = bytes_flashed >= data.len();
        log::debug!(target: "FLASH", "[Sequence {}/{}] Ending transfer", i + 1, total_seqs);
        sequence::end(c, &pit_entry, OdinInt::from(sequence_len), is_last_sequence)?;
        log::debug!(target: "FLASH", "[Sequence {}/{}] OK", i + 1, total_seqs);
    }
    log::info!(target: "FLASH", "Flash OK");

    return Ok(());
}

fn div_up(a: usize, b: usize) -> usize {
    (a + (b - 1)) / b
}

/// Tell the target how much data to expect in total.
/// TODO: Make work for multiple files (requires reworking flash functionality to accept all at once)
fn set_total_size(c: &mut Box<dyn Communicator>, data: &[u8]) -> Result<()> {
    // TODO: Unclear whether proto version 0 supports this, might need to be conditional
    log::info!(target: "FLASH", "Telling target to expect {} bytes total", data.len());
    let p = OdinCmdPacket::with_u64_arg(
        OdinCmd::SessionStart,
        OdinInt::from(SET_TOTAL_SIZE),
        data.len().try_into().unwrap(),
    );
    p.send(c)?;

    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::SessionStart {
        return Err(
            DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::SessionStart, resp.cmd).into(),
        );
    }
    return Ok(());
}

/// Tell the target we'd like to start transferring.
fn start(c: &mut Box<dyn Communicator>) -> Result<()> {
    log::debug!(target: "FLASH", "Sending start sequence");
    let p = OdinCmdPacket::with_1_arg(OdinCmd::Flash, OdinInt::from(FLASH_CMD_BEGIN_FLASH));
    p.send(c)?;

    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::Flash {
        return Err(DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::Flash, resp.cmd).into());
    }
    log::debug!(target: "FLASH", "Start sequence sent OK");
    return Ok(());
}
