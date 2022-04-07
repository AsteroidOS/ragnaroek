//! This module implements deserialization/serialization (TODO)
//! for the Samsung PIT partition file format.

use crate::download_protocol::OdinInt;

mod deserialize;
#[cfg(test)]
mod deserialize_test;
mod error;
pub use error::PitError;

const PIT_MAGIC: [u8; 4] = [0x76, 0x98, 0x34, 0x12];
const PIT_ENTRY_SIZE: usize = 132;

#[derive(Debug, Clone, PartialEq)]
pub struct PitEntry {
    pub pit_type: PitType,
    pub pit_device_type: PitDeviceType,
    pub pit_id: PitIdentifier,
    pub pit_attributes: Vec<PitAttribute>,
    pub pit_update_attributes: Vec<PitUpdateAttribute>,
    pub block_size_or_offset: OdinInt,
    pub block_count: OdinInt,
    pub file_offset: OdinInt,
    pub file_size: OdinInt,
    pub partition_name: String,
    pub flash_filename: String,
    pub fota_filename: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pit {
    entries: Vec<PitEntry>,
    // For the iterator
    idx: usize,
}

impl Iterator for Pit {
    type Item = PitEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx > (self.entries.len() - 1) {
            return None;
        }

        let next = self.entries[self.idx].clone();
        self.idx = self.idx + 1;
        return Some(next);
    }
}

impl Pit {
    pub fn get_entry_by_name(&self, name: &str) -> Option<PitEntry> {
        for e in self.clone().into_iter() {
            if e.partition_name == name {
                return Some(e);
            }
        }
        return None;
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PitType {
    Other = 0x00,
    Modem = 0x01,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PitDeviceType {
    OneNand = 0x00,
    File = 0x01,
    Mmc = 0x02,
    All = 0x03,
}

impl Into<OdinInt> for PitDeviceType {
    fn into(self) -> OdinInt {
        use PitDeviceType::*;
        match self {
            OneNand => OdinInt::from(0x00),
            File => OdinInt::from(0x01),
            Mmc => OdinInt::from(0x02),
            All => OdinInt::from(0x03),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PitAttribute {
    Write = 0x01,
    Stl = 0x02,
    Bml = 0x04,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PitUpdateAttribute {
    Fota = 0x01,
    Secure = 0x02,
}

type PitIdentifier = OdinInt;
