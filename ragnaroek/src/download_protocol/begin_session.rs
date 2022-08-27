use super::*;
use crate::Result;

use crate::comms::Communicator;

const BEGIN_SESSION: u32 = 0x00;
const SET_PACKET_SIZE: u32 = 0x05;

/// Maximum number of packets permitted in one flashing sequence by default.
const DEFAULT_MAX_SEQ_PACKETS: u32 = 800;
/// Maximum size of a single packet in a flashing sequence by default.
const DEFAULT_MAX_PACKET_SIZE: u32 = 128 * 1024; // 128KiB
/// Known protocol versions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProtoVersion {
    V0,
    V3,
    V4,
    V5,
}
/// Session parameters negotiated with the target.
#[derive(Clone, Copy, Debug)]
pub struct SessionParams {
    /// Whether the target supports receiving raw LZ4-compressed blocks.
    pub supports_compression: bool,
    /// Negotiated protocol version.
    pub proto_version: ProtoVersion,
    /// Negotiated flash packet size.
    pub max_packet_size: u32,
    /// Negotiated maximum number of packets in flash sequence.
    pub max_seq_packets: u32,
    /// Negotiated maximum sequence total size in bytes.
    pub max_seq_size_bytes: u32,
}

fn negotiate_packet_size(c: &mut Box<dyn Communicator>, v: ProtoVersion) -> Result<OdinInt> {
    // This fixed size should be safe for proto version 0
    if v == ProtoVersion::V0 {
        return Ok(OdinInt::from(DEFAULT_MAX_PACKET_SIZE));
    }

    // Other versions support negotiation (and may, in fact, require it)
    // In future, we could get more creative here.
    let p = OdinCmdPacket::with_2_args(
        OdinCmd::SessionStart,
        OdinInt::from(SET_PACKET_SIZE),
        OdinInt::from(DEFAULT_MAX_PACKET_SIZE),
    );
    p.send(c)?;

    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::SessionStart {
        return Err(
            DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::SessionStart, resp.cmd).into(),
        );
    }
    return Ok(resp.arg);
}

fn negotiate_max_seq_packets(c: &mut Box<dyn Communicator>, v: ProtoVersion) -> Result<OdinInt> {
    // Currently, it seems like there is no known mechanism to negotiate this.
    let _ = c;
    let _ = v;
    return Ok(OdinInt::from(DEFAULT_MAX_SEQ_PACKETS));
}

/// Tries to guess the supported protocol version of the target and whether it supports compression.
/// Not sure whether this always works, because the version negotiation schema is super stupid.
/// Returns `Err(())` if version couldn't be determined or is one ragnaroek doesn't know about.
fn guess_version_and_compression(c: &mut Box<dyn Communicator>) -> Result<(ProtoVersion, bool)> {
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
            return Ok((ProtoVersion::V0, false));
        }

        // All other bootloader versions return (<Our Protocol Version> << 16) | 0x0 if BL version is newer
        // Increment and try again
        if ((resp.arg.inner << 16) | 0x00) == curr_version {
            curr_version += 1;
            continue;
        }

        // If compression is supported, this bit is set
        let supports_compression: bool = (resp.arg.inner & 0x8000) > 0;
        match curr_version {
            3 => return Ok((ProtoVersion::V3, supports_compression)),
            4 => return Ok((ProtoVersion::V4, supports_compression)),
            5 => return Ok((ProtoVersion::V5, supports_compression)),
            _ => {
                return Err(DownloadProtocolError::UnknownProtoVersion(curr_version.into()).into())
            }
        };
    }
}

/// Begins a session with a target.
pub(crate) fn begin_session(c: &mut Box<dyn Communicator>) -> Result<SessionParams> {
    let (proto_version, supports_compression) = guess_version_and_compression(c).unwrap();

    let max_packet_size = negotiate_packet_size(c, proto_version)?.inner;
    let max_seq_packets = negotiate_max_seq_packets(c, proto_version)?.inner;

    let params = SessionParams {
        supports_compression,
        proto_version,
        max_packet_size,
        max_seq_packets,
        max_seq_size_bytes: max_seq_packets * max_packet_size,
    };
    println!("Session params: {:?}", params);

    return Ok(params);
}
