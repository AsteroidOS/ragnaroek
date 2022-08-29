use super::send_packet;
use super::Bitness;
use crate::{Communicator, Result};

const DATAXFER: &[u8] = &[b'D', b'a', b'T', b'a', b'X', b'f', b'E', b'r', b'\0'];
const TRANSFER_MAX_SIZE: usize = 0x80000; // 512KiB

/// Dump target memory in upload mode.
///
/// Note that while u64's are accepted here,
/// if the target is 32 bit values that are too large will be rejected with an error.
pub fn transfer(
    c: &mut Box<dyn Communicator>,
    bitness: Bitness,
    start_addr: u64,
    end_addr: u64,
) -> Result<Vec<u8>> {
    unimplemented!();
    /*
    // TODO: Sanity checks
    // TODO: start_addr > end_addr
    // TODO: Bitness::ThirtyTwo && start_addr > u32::MAX
    // TODO: Bitness::ThirtyTwo && end_addr > u32::MAX

    // Because the amount of data that can be transferred at once is limited,
    // we have to split the requested range up.
    // TODO: This prevents dumping more than 4 gigs on 32-bit platforms. Fix that.
    let start_addr_usize: usize = start_addr.try_into().unwrap();
    let end_addr_usize: usize = end_addr.try_into().unwrap();
    let size: usize = end_addr_usize - start_addr_usize;
    let mut data: Vec<u8> = Vec::new();
    // TODO: Not a good idea to keep the entire buffer in memory due to RAM constraints.
    data.reserve(size);

    let mut chunk_start_addr: u64 = start_addr + (data.len() as u64);
    let remaining = end_addr_usize - size;
    while data.len() < size {
        let remaining = end_addr_usize - size;
    }

    // Actually receive the transfer
    match c.recv_exact(size) {
        Err(e) => return Err(Error::TransferError(TransferError::IoError(e))),
        Ok(data) => return Ok(data),
    }
    */
}

/// Transfer the requested chunk of memory.
fn transfer_chunk(
    c: &mut Box<dyn Communicator>,
    bitness: Bitness,
    start_addr: u64,
    end_addr: u64,
) -> Result<Vec<u8>> {
    // Tell target which range we're interested in
    transfer_configure(c, bitness, start_addr, end_addr)?;
    // Tell target to start transfer
    send_packet(c, DATAXFER)?;

    return Ok(Vec::new());
}

/// Tell the target which memory range to dump.
/// Note that the size should not exceed TRANSFER_MAX_SIZE.
fn transfer_configure(
    c: &mut Box<dyn Communicator>,
    bitness: Bitness,
    start_addr: u64,
    end_addr: u64,
) -> Result<()> {
    // TODO: Check for exceedance of size
    match bitness {
        Bitness::ThirtyTwo => {
            let start_addr: [u8; 4] = u32::to_le_bytes(start_addr.try_into().unwrap());
            send_packet(c, &start_addr)?;
            let end_addr: [u8; 4] = u32::to_le_bytes(end_addr.try_into().unwrap());
            send_packet(c, &end_addr)?;
        }
        Bitness::SixtyFour => {
            let start_addr: [u8; 8] = u64::to_le_bytes(start_addr);
            send_packet(c, &start_addr)?;
            let end_addr: [u8; 8] = u64::to_le_bytes(end_addr);
            send_packet(c, &end_addr)?;
        }
    }

    return Ok(());
}
