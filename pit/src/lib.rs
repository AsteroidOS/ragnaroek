//! This crate implements deserialization/serialization (TODO)
//! for the Samsung PIT partition file format.

#![allow(clippy::needless_return)]

mod deserialize;
#[cfg(test)]
mod deserialize_test;
mod error;
mod pit_entry;

#[cfg(feature = "serde")]
use serde::Serialize;

pub use either::Either;

pub use error::PitError;
pub use pit_entry::*;

// Re-export the tabled crate because some features require free functions from it.
#[cfg(feature = "tabled")]
pub use tabled;

const PIT_MAGIC: [u8; 4] = [0x76, 0x98, 0x34, 0x12];
const PIT_HEADER_SIZE: usize = 28;
const PIT_ENTRY_SIZE: usize = 132;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Pit(pub Either<PitV1, PitV2>);

impl Pit {
    pub(crate) fn from_v1(p: PitV1) -> Pit {
        return Pit(Either::Left(p));
    }
    pub(crate) fn from_v2(p: PitV2) -> Pit {
        return Pit(Either::Right(p));
    }

    pub fn get_entry_by_name(&self, name: &str) -> Option<Either<PitEntryV1, PitEntryV2>> {
        match &self.0 {
            Either::Left(s) => {
                let entry = s.get_entry_by_name(name)?;
                return Some(Either::Left(entry));
            }
            Either::Right(s) => {
                let entry = s.get_entry_by_name(name)?;
                return Some(Either::Right(entry));
            }
        }
    }

    pub fn gang_name(&self) -> String {
        match &self.0 {
            Either::Left(s) => {
                return s.gang_name.clone();
            }
            Either::Right(s) => {
                return s.gang_name.clone();
            }
        }
    }

    pub fn project_name(&self) -> String {
        match &self.0 {
            Either::Left(s) => {
                return s.project_name.clone();
            }
            Either::Right(s) => {
                return s.project_name.clone();
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// A version 1 PIT file.
pub struct PitV1 {
    /// Usually "COM_TAR2"
    pub gang_name: String,
    /// Usually the device's model number
    pub project_name: String,
    entries: Vec<PitEntryV1>,
    // For the iterator
    idx: usize,
}

impl Iterator for PitV1 {
    type Item = PitEntryV1;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx > (self.entries.len() - 1) {
            return None;
        }

        let next = self.entries[self.idx].clone();
        self.idx += 1;
        return Some(next);
    }
}

impl PitV1 {
    pub fn get_entry_by_name(&self, name: &str) -> Option<PitEntryV1> {
        return self.clone().find(|e| e.partition_name == name);
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// A version 2 PIT file.
pub struct PitV2 {
    /// Usually "COM_TAR2"
    pub gang_name: String,
    /// Usually the device's model number
    pub project_name: String,
    entries: Vec<PitEntryV2>,
    // For the iterator
    idx: usize,
}

impl Iterator for PitV2 {
    type Item = PitEntryV2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx > (self.entries.len() - 1) {
            return None;
        }

        let next = self.entries[self.idx].clone();
        self.idx += 1;
        return Some(next);
    }
}

impl PitV2 {
    pub fn get_entry_by_name(&self, name: &str) -> Option<PitEntryV2> {
        return self.clone().into_iter().find(|e| e.partition_name == name);
    }
}
