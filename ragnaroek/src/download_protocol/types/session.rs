use either::Either;
use pit::*;

use super::super::begin_session::*;
use super::super::download_pit::*;
pub use super::super::end_session::ActionAfter;
use super::super::end_session::*;
use super::super::flash::*;
use super::super::magic_handshake::*;
use crate::download_protocol::*;
use crate::Communicator;
use crate::Result;

const BEGIN_SESSION: u32 = 0x00;
const T_FLASH: u32 = 0x08;

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

    /// Enter T-Flash download mode (write to microSD card).
    /// Call this after `begin` and before `end`.
    pub fn enable_tflash(&mut self) -> Result<()> {
        log::debug!(target: "SESS", "Enabling T-Flash mode");
        let p = OdinCmdPacket::with_2_args(
            OdinCmd::SessionStart,
            OdinInt::from(BEGIN_SESSION),
            OdinInt::from(T_FLASH),
        );
        p.send(&mut self.c)?;

        let resp = OdinCmdReply::read(&mut self.c)?;
        if resp.cmd != OdinCmd::SessionStart {
            return Err(
                DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::SessionStart, resp.cmd).into(),
            );
        }

        // Anything other than 0x00 indicates failure.
        if resp.arg != OdinInt::from(0x00) {
            return Err(
                DownloadProtocolError::UnexpectedOdinCmdArg(OdinInt::from(0x00), resp.arg).into(),
            );
        }
        log::debug!(target: "SESS", "T-Flash mode enabled");
        return Ok(());
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
    ///
    /// `cb` is an optional callback, called after each file part is transferred with the number of bytes transferred since the last call.
    pub fn flash(
        &mut self,
        data: &[u8],
        pit_entry: Either<PitEntryV1, PitEntryV2>,
        cb: &mut Option<&mut impl FnMut(u64)>,
    ) -> Result<()> {
        return flash(&mut self.c, self.params, data, pit_entry, cb);
    }

    /// The top-level flash function.
    ///
    /// It calls `flash()` for each component of the Odin TAR file.
    ///
    /// `cb` is a callback for e.g. displaying a progress bar.
    pub fn flash_odintar(
        &mut self,
        rdr: &mut dyn SeekableReader,
        pit: Pit,
        // TODO: Make this filename-aware. For now, it's just called for each file in the archive.
        cb: &mut Option<&mut impl FnMut(u64)>,
    ) -> Result<()> {
        return flash_odintar(&mut self.c, self.params, rdr, pit, cb);
    }

    /// Factory reset user data on the target.
    pub fn factory_reset(&mut self) -> Result<()> {
        return factory_reset(&mut self.c);
    }
}
