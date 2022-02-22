//! Module implementing memory dumping via upload mode.
//! Heavily based on https://github.com/bkerler/sboot_dump.

mod end_session;
mod error;
mod handshake;
mod probe;
mod transfer;

pub use end_session::end_session;
pub use error::*;
pub use handshake::handshake;
pub use probe::*;

use crate::Communicator;
use crate::Result;

/// Length all packets should be padded to.
const PACKET_LEN: usize = 1024;

/// Targets can have a different bitness, which changes the length of memory addresses.
#[derive(Clone, Copy, Debug)]
pub enum Bitness {
    /// Target has 32-bit addresses
    ThirtyTwo,
    /// Target has 64-bit addresses
    SixtyFour,
}

/// Sends the given packet to the target.
/// Adds padding if needed.
fn send_packet(c: &mut Box<dyn Communicator>, data: &[u8]) -> Result<()> {
    let mut padded: Vec<u8> = Vec::new();
    padded.resize(PACKET_LEN, 0);
    for (i, byte) in data.iter().enumerate() {
        padded[i] = *byte;
    }
    c.send(&padded)?;

    return Ok(());
}

/// Dump the given memory range.
pub fn dump(c: &mut Box<dyn Communicator>, start_addr: u64, end_addr: u64) -> Result<Vec<u8>> {
    unimplemented!();
    handshake::handshake(c)?;

    let mut data: Vec<u8> = Vec::new();

    end_session::end_session(c)?;
    return Ok(data);
}
