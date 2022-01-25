//! This module implements deserialization/serialization (TODO)
//! for the Samsung PIT partition file format.

use crate::Result;

#[derive(Debug, Clone, PartialEq)]
pub struct Pit {
	data: Vec<u8> // TODO: Parse
}

impl Pit {
	/// Obtain a PIT structure by parsing it's binary representation.
	pub fn deserialize(data: &[u8]) -> Result<Pit> {
		return Ok(Pit{data: Vec::from(data)});
	}
}