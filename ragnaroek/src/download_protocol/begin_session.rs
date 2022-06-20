use super::*;
use crate::Result;

use crate::comms::Communicator;

const BEGIN_SESSION: u32 = 0x00;
const SET_TOTAL_SIZE: u32 = 0x02;
const SET_PART_SIZE: u32 = 0x05;

/// Known protocol versions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ProtoVersion {
    V0,
    V3,
    V4,
    V5,
}
/// Session parameters negotiated with the target.
#[derive(Clone, Copy, Debug)]
pub(crate) struct SessionParams {
    /// Whether the target supports receiving raw LZ4-compressed blocks.
    supports_compression: bool,
    /// Negotiated protocol version.
    proto_version: ProtoVersion,
}

/// Tries to guess the supported protocol version of the target and whether it supports compression.
/// Not sure whether this always works, because the version negotiation schema is super stupid.
/// Returns `Err(())` if version couldn't be determined or is one ragnaroek doesn't know about.
fn guess_session_params(c: &mut Box<dyn Communicator>) -> Result<SessionParams> {
    let mut curr_version = 0;
    loop {
        let p = OdinCmdPacket::with_2_args(
            OdinCmd::SessionStart,
            OdinInt::from(BEGIN_SESSION),
            OdinInt::from(curr_version),
        );
        p.send(c)?;

        let resp = OdinCmdReply::read(c)?;
        if resp.cmd != OdinCmd::SessionStart {
            return Err(
                DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::SessionStart, resp.cmd).into(),
            );
        }

        // It's hard to differentiate between a target not supporting the given version and replying nonsense.
        // To work around that, let's assume that Samsung won't release >100 protocol versions.
        if curr_version > 100 {
            // TODO: Cleanup
            panic!("Target responded with unexpected {:?}", resp.arg.inner);
        }

        // Version 0 is special, because of course it is
        if resp.arg == OdinInt::from(0x20000) {
            return Ok(SessionParams {
                supports_compression: false,
                proto_version: ProtoVersion::V0,
            });
        }

        // All other bootloader versions return (<Our Protocol Version> << 16) | 0x0 if BL version is newer
        // Increment and try again
        if ((resp.arg.inner << 16) | 0x00) == curr_version {
            curr_version += 1;
            continue;
        }

        // We probably guessed correctly if we're here
        // If compression is supported, this bit is set
        let supports_compression: bool = (resp.arg.inner & 0x8000) > 0;
        match curr_version {
            3 => {
                return Ok(SessionParams {
                    supports_compression,
                    proto_version: ProtoVersion::V3,
                })
            }
            4 => {
                return Ok(SessionParams {
                    supports_compression,
                    proto_version: ProtoVersion::V4,
                })
            }
            5 => {
                return Ok(SessionParams {
                    supports_compression,
                    proto_version: ProtoVersion::V5,
                })
            }
            _ => {
                return Err(DownloadProtocolError::UnknownProtoVersion(curr_version.into()).into())
            }
        };
    }
}

/// Begins a session with a target.
pub fn begin_session(c: &mut Box<dyn Communicator>) -> Result<()> {
    let session_params = guess_session_params(c).unwrap();
    println!("Session params: {:?}", session_params);
    let p = OdinCmdPacket::with_2_args(
        OdinCmd::SessionStart,
        OdinInt::from(SET_PART_SIZE),
        OdinInt::from(131072),
    );
    p.send(c)?;

    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::SessionStart {
        return Err(
            DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::SessionStart, resp.cmd).into(),
        );
    };

    // Set the total length. Not sure if this refers to the file or what.
    // Not supported in protocol version 0.
    if session_params.proto_version != ProtoVersion::V0 {
        let p = OdinCmdPacket::with_2_args(
            OdinCmd::SessionStart,
            OdinInt::from(SET_TOTAL_SIZE),
            OdinInt::from(0x4a7fa500),
        );
        p.send(c)?;

        let resp = OdinCmdReply::read(c)?;
        if resp.cmd != OdinCmd::SessionStart {
            return Err(
                DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::SessionStart, resp.cmd).into(),
            );
        }
    }

    return Ok(());
}
