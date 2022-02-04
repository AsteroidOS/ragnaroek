use super::*;

use crate::pit::PitEntry;
use crate::Communicator;

const FLASH_FLAG_FLASH: u32 = 0x00;
const FLASH_FLAG_END: u32 = 0x03;

const CHUNK_MAX_SIZE: usize = 131072; // 128KiB

pub fn flash(c: &mut Box<dyn Communicator>, mut data: &[u8]) -> Result<()> {
    flash_initiate(c)?;
    while data.len() > 0 {
        data = flash_chunk(c, data)?;
    }
    flash_finish(c)?;

    return Ok(());
}

pub fn flash_initiate(c: &mut Box<dyn Communicator>) -> Result<()> {
    // Tell target that we want to flash
    let p1 = OdinCmdPacket {
        kind: OdinCmd::Flash,
        arg1: OdinInt::from(FLASH_FLAG_FLASH),
        arg2: None,
    };
    p1.send(c)?;

    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::Flash {
        panic!(
            "Target sent unexpected Odin command in reply: {:?}",
            resp.cmd
        );
    }
    return Ok(());
}

pub fn flash_chunk<'a>(c: &mut Box<dyn Communicator>, data: &'a [u8]) -> Result<&'a [u8]> {
    // Slice off either the max chunk size or the remaining data, whichever is smaller
    let size = std::cmp::min(data.len(), CHUNK_MAX_SIZE);
    let chunk = &data[0..size];
    // FIXME: Off-by-one?
    let data_remaining = &data[size..];

    c.send(chunk)?;

    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::ChunkTransferOk {
        panic!(
            "Target sent unexpected Odin command in reply: {:?}",
            resp.cmd
        );
    }
    return Ok(&data_remaining);
}

pub fn flash_finish(c: &mut Box<dyn Communicator>) -> Result<()> {
    // Tell target that we are done
    let p1 = OdinCmdPacket {
        kind: OdinCmd::Flash,
        arg1: OdinInt::from(FLASH_FLAG_END),
        arg2: None,
    };
    p1.send(c)?;

    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::Flash {
        panic!(
            "Target sent unexpected Odin command in reply: {:?}",
            resp.cmd
        );
    }
    return Ok(());
}
