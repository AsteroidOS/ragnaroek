use super::*;
use crate::Result;

use crate::comms::Communicator;

const BEGIN_SESSION: u32 = 0x00;
const SET_PACKET_SIZE: u32 = 0x05;

/// Maximum number of file parts permitted in one flashing sequence for protocol version 1.
const V1_MAX_SEQ_PARTS: u32 = 800;
/// Maximum size of a single packet in a flashing sequence for protocol version 1.
const V1_MAX_FILE_PART_SIZE: u32 = 128 * 1024; // 128KiB
/// Maximum number of file parts permitted in one flashing sequence for protocol version >1.
const V2PLUS_MAX_SEQ_PARTS: u32 = 30;
/// Maximum size of a single packet in a flashing sequence for protocol version >1.
const V2PLUS_MAX_FILE_PART_SIZE: u32 = 1 * 1024 * 1024; // 1MiB
/// Highest protocol version we support.
const MAX_PROTO_VERSION: u32 = 0x04;

/// Known protocol versions.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProtoVersion {
    V1,
    V3,
    V4,
}
/// Session parameters negotiated with the target.
#[derive(Clone, Copy, Debug)]
pub struct SessionParams {
    /// Whether the target supports receiving raw LZ4-compressed blocks.
    pub supports_compression: bool,
    /// Negotiated protocol version.
    pub proto_version: ProtoVersion,
    /// Negotiated file part size.
    pub max_file_part_size: u32,
    /// Negotiated maximum number of file parts in flash sequence.
    pub max_seq_file_parts: u32,
    /// Negotiated maximum sequence total size in bytes.
    pub max_seq_size_bytes: u32,
}

fn negotiate_packet_size(c: &mut Box<dyn Communicator>, v: ProtoVersion) -> Result<OdinInt> {
    // This fixed size should be safe for proto version 0
    if v == ProtoVersion::V1 {
        log::trace!(target: "SESS", "Setting packet size supported by version 1: {}", V1_MAX_FILE_PART_SIZE);
        return Ok(OdinInt::from(V1_MAX_FILE_PART_SIZE));
    }

    // Other versions support negotiation (and may, in fact, require it)
    // In future, we could get more creative here.
    log::trace!(target: "SESS", "Version is newer than 1, negotiating packet size {}", V2PLUS_MAX_FILE_PART_SIZE);
    let p = OdinCmdPacket::with_2_args(
        OdinCmd::SessionStart,
        OdinInt::from(SET_PACKET_SIZE),
        OdinInt::from(V2PLUS_MAX_FILE_PART_SIZE),
    );
    p.send(c)?;

    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::SessionStart {
        return Err(
            DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::SessionStart, resp.cmd).into(),
        );
    }
    if resp.arg != OdinInt::from(0) {
        return Err(DownloadProtocolError::UnexpectedOdinCmdArg(OdinInt::from(0), resp.arg).into());
    }
    return Ok(OdinInt::from(V2PLUS_MAX_FILE_PART_SIZE));
}

fn get_max_seq_file_parts(v: ProtoVersion) -> Result<OdinInt> {
    // Currently, it seems like there is no known mechanism to negotiate this.
    // However, according to samsung-loki there's a safe defualt to use here.
    use ProtoVersion::*;
    match v {
        V1 => return Ok(OdinInt::from(V1_MAX_SEQ_PARTS)),
        _ => return Ok(OdinInt::from(V2PLUS_MAX_SEQ_PARTS)),
    }
}

/// Determines the supported protocol version of the target and whether it supports compression.
/// Returns `Err(())` if version couldn't be determined or is one ragnaroek doesn't know about.
fn determine_version_and_compression(
    c: &mut Box<dyn Communicator>,
) -> Result<(ProtoVersion, bool)> {
    let p = OdinCmdPacket::with_2_args(
        OdinCmd::SessionStart,
        OdinInt::from(BEGIN_SESSION),
        OdinInt::from(MAX_PROTO_VERSION),
    );
    p.send(c)?;

    let resp = OdinCmdReply::read(c)?;
    if resp.cmd != OdinCmd::SessionStart {
        return Err(
            DownloadProtocolError::UnexpectedOdinCmd(OdinCmd::SessionStart, resp.cmd).into(),
        );
    }
    let bl_version = (resp.arg.inner & 0xF0000) >> 16;

    // It's hard to differentiate between a target not supporting the given version and replying nonsense.
    // To work around that, let's assume that Samsung won't release >100 protocol versions.
    if bl_version > 100 {
        // TODO: Cleanup
        panic!("Target responded with bogus version {:?}", resp.arg.inner);
    }

    // Version 1 is special, because of course it is
    if resp.arg == OdinInt::from(0) {
        return Ok((ProtoVersion::V1, false));
    }

    // If compression is supported, this bit is set
    let supports_compression: bool = (resp.arg.inner & 0x8000) > 0;

    match bl_version {
        1 => return Ok((ProtoVersion::V1, supports_compression)),
        3 => return Ok((ProtoVersion::V3, supports_compression)),
        4 => return Ok((ProtoVersion::V4, supports_compression)),
        _ => return Err(DownloadProtocolError::UnknownProtoVersion(bl_version.into()).into()),
    };
}

/// Begins a session with a target.
pub(crate) fn begin_session(c: &mut Box<dyn Communicator>) -> Result<SessionParams> {
    log::debug!(target: "SESS", "Beginning session");
    let (proto_version, supports_compression) = determine_version_and_compression(c).unwrap();

    let max_file_part_size = negotiate_packet_size(c, proto_version)?.inner;
    let max_seq_file_parts = get_max_seq_file_parts(proto_version)?.inner;

    let params = SessionParams {
        supports_compression,
        proto_version,
        max_file_part_size,
        max_seq_file_parts,
        max_seq_size_bytes: max_seq_file_parts * max_file_part_size,
    };
    log::debug!(target: "SESS", "Negotiated session params: {:?}", params);

    return Ok(params);
}
