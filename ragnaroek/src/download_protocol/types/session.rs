use either::Either;
use pit::*;

use super::super::begin_session::*;
use super::super::download_pit::*;
pub use super::super::end_session::ActionAfter;
use super::super::end_session::*;
use super::super::flash::*;
use super::super::magic_handshake::*;
use crate::Communicator;
use crate::Result;

/// This module's main type.
/// Manages the communications lifecycle with the target for the download protocol.
pub struct Session {
    c: Box<dyn Communicator>,
    /// Session parameters, such as sizes of various transfers and the protocol version.
    pub params: SessionParams,
}

// The actual logic is much too complex to include it here.
// Instead, these are thin RAII wrappers around internal functions.
impl Session {
    /// Create a new `Session` and negotiate connection parameters with the target.
    /// Consumes the `Communicator` to enforce exclusive access.
    /// If the `Communicator` has been used to send data to the target before, the behavior of target is undefined.
    pub fn begin(mut c: Box<dyn Communicator>) -> Result<Self> {
        magic_handshake(&mut c)?;
        let params = begin_session(&mut c)?;
        return Ok(Session { c, params });
    }

    /// End the `Session` and do cleanup.
    pub fn end(mut self, after: ActionAfter) -> Result<()> {
        end_session(&mut self.c, after)?;
        return Ok(());
    }

    /// Download partitioning data from the target. Does not parse or validate the data.
    pub fn download_pit(&mut self, p: SessionParams) -> Result<Vec<u8>> {
        return download_pit(&mut self.c, p);
    }

    /// Flash a file to the target.
    pub fn flash(&mut self, data: &[u8], pit_entry: Either<PitEntryV1, PitEntryV2>) -> Result<()> {
        return flash(&mut self.c, self.params, data, pit_entry);
    }
    /// Factory reset user data on the target.
    pub fn factory_reset(&mut self) -> Result<()> {
        return factory_reset(&mut self.c);
    }
}
