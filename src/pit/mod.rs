//! This module implements deserialization/serialization (TODO)
//! for the Samsung PIT partition file format.

use crate::download_protocol::OdinInt;

use core::fmt;

use tabled::Tabled;

mod deserialize;
#[cfg(test)]
mod deserialize_test;
mod error;
pub use error::PitError;

const PIT_MAGIC: [u8; 4] = [0x76, 0x98, 0x34, 0x12];
const PIT_ENTRY_SIZE: usize = 132;

#[derive(Debug, Clone, PartialEq, Tabled)]
pub struct PitEntry {
    pub pit_type: PitType,
    pub pit_device_type: PitDeviceType,
    pub pit_id: PitIdentifier,
    #[tabled(display_with = "display_pit_attributes")]
    pub pit_attributes: Vec<PitAttribute>,
    #[tabled(display_with = "display_pit_update_attributes")]
    pub pit_update_attributes: Vec<PitUpdateAttribute>,
    pub block_size_or_offset: OdinInt,
    pub block_count: OdinInt,
    pub file_offset: OdinInt,
    pub file_size: OdinInt,
    pub partition_name: String,
    pub flash_filename: String,
    pub fota_filename: String,
}

fn display_pit_attributes(attrs: &Vec<PitAttribute>) -> String {
    let mut s = String::new();
    for a in attrs {
        s.push_str(&format!("{}\n", a));
    }
    return s;
}

fn display_pit_update_attributes(attrs: &Vec<PitUpdateAttribute>) -> String {
    let mut s = String::new();
    for a in attrs {
        s.push_str(&format!("{}\n", a));
    }
    return s;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pit {
    /// Usually "COM_TAR2"
    pub gang_name: String,
    /// Usually the device's model number
    pub project_name: String,
    /// Version of the PIT file. Not sure if related to the download mode protocol version.
    pub proto_version: OdinInt,
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

#[derive(Debug, Copy, Clone, PartialEq, Tabled)]
pub enum PitType {
    Other = 0x00,
    Modem = 0x01,
}

impl fmt::Display for PitType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PitType::Other => write!(f, "Phone/AP"),
            PitType::Modem => write!(f, "Modem/CP"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Tabled)]
pub enum PitDeviceType {
    Nand = 0x01,
    Emmc = 0x02,
    Spi = 0x03,
    Ide = 0x04,
    NandX16 = 0x05,
    Nor = 0x06,
    NandWB1 = 0x07,
    Ufs = 0x08,
}

impl fmt::Display for PitDeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use PitDeviceType::*;
        match self {
            Nand => write!(f, "NAND"),
            Emmc => write!(f, "EMCC"),
            Spi => write!(f, "SPI"),
            Ide => write!(f, "IDE"),
            NandX16 => write!(f, "NANDX16"),
            Nor => write!(f, "NOR"),
            NandWB1 => write!(f, "NANDWB1"),
            Ufs => write!(f, "UFS"),
        }
    }
}

impl Into<OdinInt> for PitDeviceType {
    fn into(self) -> OdinInt {
        use PitDeviceType::*;
        match self {
            Nand => OdinInt::from(0x01),
            Emmc => OdinInt::from(0x02),
            Spi => OdinInt::from(0x03),
            Ide => OdinInt::from(0x04),
            NandX16 => OdinInt::from(0x05),
            Nor => OdinInt::from(0x06),
            NandWB1 => OdinInt::from(0x07),
            Ufs => OdinInt::from(0x08),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Tabled)]
pub enum PitAttribute {
    Write = 0x01,
    Stl = 0x02,
    Bml = 0x04,
}

impl fmt::Display for PitAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PitAttribute::Write => write!(f, "Writable"),
            PitAttribute::Stl => write!(f, "STL"),
            PitAttribute::Bml => write!(f, "BML"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Tabled)]
pub enum PitUpdateAttribute {
    Fota = 0x01,
    Secure = 0x02,
}

impl fmt::Display for PitUpdateAttribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PitUpdateAttribute::Fota => write!(f, "FOTA"),
            PitUpdateAttribute::Secure => write!(f, "Secure"),
        }
    }
}

type PitIdentifier = OdinInt;
