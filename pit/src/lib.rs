//! This crate implements deserialization/serialization (TODO)
//! for the Samsung PIT partition file format.

mod deserialize;
#[cfg(test)]
mod deserialize_test;
mod error;
mod pit_entry;

pub use error::PitError;
pub use pit_entry::*;

// Re-export the tabled crate because some features require free functions from it.
#[cfg(feature = "tabled")]
pub use tabled;

const PIT_MAGIC: [u8; 4] = [0x76, 0x98, 0x34, 0x12];
const PIT_ENTRY_SIZE: usize = 132;

#[derive(Debug, Clone, PartialEq)]
pub struct Pit {
    /// Usually "COM_TAR2"
    pub gang_name: String,
    /// Usually the device's model number
    pub project_name: String,
    /// Version of the PIT file. Not sure if related to the download mode protocol version.
    pub proto_version: u32,
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
