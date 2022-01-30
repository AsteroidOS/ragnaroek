//! This module implements deserialization/serialization (TODO)
//! for the Samsung PIT partition file format.

use crate::protocol::OdinInt;

mod deserialize;
mod error;
pub use error::PitError;

const PIT_MAGIC: [u8; 4] = [0x76, 0x98, 0x34, 0x12];
const PIT_ENTRY_SIZE: usize = 132;

#[derive(Debug, Clone, PartialEq)]
pub struct PitEntry {
    pit_type: PitType,
    pit_device_type: PitDeviceType,
    pit_id: PitIdentifier,
    pit_attributes: Vec<PitAttribute>,
    pit_update_attributes: Vec<PitUpdateAttribute>,
    block_size_or_offset: OdinInt,
    block_count: OdinInt,
    file_offset: OdinInt,
    file_size: OdinInt,
    partition_name: String,
    flash_filename: String,
    fota_filename: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pit {
    entries: Vec<PitEntry>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum PitType {
    Other = 0x00,
    Modem = 0x01,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum PitDeviceType {
    OneNand = 0x00,
    File = 0x01,
    Mmc = 0x02,
    All = 0x03,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum PitAttribute {
    Write = 0x01,
    Stl = 0x02,
    Bml = 0x04,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum PitUpdateAttribute {
    Fota = 0x01,
    Secure = 0x02,
}

type PitIdentifier = OdinInt;
