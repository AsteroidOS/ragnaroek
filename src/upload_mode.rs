use crate::error::TransferError;
use crate::Communicator;
use crate::{Error, Result};

const PREAMBLE: &[u8] = &[b'P', b'r', b'E', b'a', b'M', b'b', b'L', b'e', b'\0'];
const DATAXFER: &[u8] = &[b'D', b'a', b'T', b'a', b'X', b'f', b'E', b'r', b'\0'];
const PACKET_LEN: usize = 1024;

/// Dump target memory in upload mode.
pub fn dump_memory(
    c: &mut Box<dyn Communicator>,
    start_addr: u32,
    end_addr: u32,
) -> Result<Vec<u8>> {
    initiate(c, start_addr, end_addr)?;
    send_padded_packet(c, DATAXFER)?; // This tells the target to actually start the transfer

    let start_addr: usize = start_addr.try_into().unwrap();
    let end_addr: usize = end_addr.try_into().unwrap();
    let size: usize = end_addr - start_addr;

    match c.recv_exact(size) {
        Err(e) => return Err(Error::TransferError(TransferError::IoError(e))),
        Ok(data) => return Ok(data),
    }
}

/// Pad the packet to a length of PACKET_LEN before sending it.
fn send_padded_packet(c: &mut Box<dyn Communicator>, data: &[u8]) -> Result<()> {
    let mut padded: Vec<u8> = Vec::new();
    padded.resize(PACKET_LEN, 0);
    for (i, byte) in data.iter().enumerate() {
        padded[i] = *byte;
    }
    c.send(&padded)?;

    return Ok(());
}

/// Tell the target how much memory we want to dump.
fn initiate(c: &mut Box<dyn Communicator>, start_addr: u32, end_addr: u32) -> Result<()> {
    send_padded_packet(c, PREAMBLE)?;
    let start_addr: [u8; 4] = u32::to_le_bytes(start_addr);
    send_padded_packet(c, &start_addr)?;
    let end_addr: [u8; 4] = u32::to_le_bytes(end_addr);
    send_padded_packet(c, &end_addr)?;
    return Ok(());
}
