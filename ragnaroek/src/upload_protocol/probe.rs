use std::ffi::CString;

use super::*;
use crate::Communicator;

use either::*;

const DEVICE_NAME_LEN: usize = 16;
const PARTITION_NAME_LEN: usize = 12;
const PROBE: &[u8] = &[b'P', b'r', b'O', b'b', b'E', b'\0'];

/// Data structure holding information the target returns about itself.
#[derive(Debug, Clone)]
pub struct ProbeTable {
    bitness: Bitness,
    device_name: String,
    /// Which of these variants to use always corellates with the bitness field.
    entries: Either<Vec<ProbeEntry32>, Vec<ProbeEntry64>>,
}

/// 32-bit probe table entry.
#[derive(Debug, Clone)]
struct ProbeEntry32 {
    partition_type: u32,
    partition_name: String,
    start_addr: u32,
    end_addr: u32,
}

/// 64-bit probe table entry.
#[derive(Debug, Clone)]
struct ProbeEntry64 {
    partition_type: u32,
    partition_name: String,
    partition_info: u64,
    start_addr: u64,
    end_addr: u64,
}

impl ProbeTable {
    fn deserialize(data: &[u8]) -> ProbeTable {
        let bitness = match data[0] {
            b'+' => Bitness::SixtyFour,
            _ => Bitness::ThirtyTwo,
        };
        // TODO: Advance data here or leave bitness as part of name?

        // TODO: Read as non-NULL terminated
        let (device_name, data) = read_string_and_advance(data, DEVICE_NAME_LEN);

        // Keep reading an unknown amount of entries
        let data = data;
        let entries = match bitness {
            Bitness::ThirtyTwo => {
                let (entries_32, _) = read_probe_entries_32_and_advance(data);
                Left(entries_32)
            }
            Bitness::SixtyFour => {
                let (entries_64, _) = read_probe_entries_64_and_advance(data);
                Right(entries_64)
            }
        };

        let table = ProbeTable {
            bitness,
            device_name,
            entries,
        };
        return table;
    }
}

/// Read as many 32-bit probe table entries as possible.
fn read_probe_entries_32_and_advance(data: &[u8]) -> (Vec<ProbeEntry32>, &[u8]) {
    let mut data = data;
    let mut entries: Vec<ProbeEntry32> = Vec::new();

    loop {
        let (entry, remaining_data) = ProbeEntry32::deserialize(data).unwrap();
        data = remaining_data;

        if (entry.start_addr == 0 && entry.end_addr == 0) || entry.start_addr < 20 {
            break;
        }

        entries.push(entry);
    }

    return (entries, data);
}

/// Read as many 64-bit probe table entries as possible.
fn read_probe_entries_64_and_advance(data: &[u8]) -> (Vec<ProbeEntry64>, &[u8]) {
    let mut data = data;
    let mut entries: Vec<ProbeEntry64> = Vec::new();

    loop {
        let (entry, remaining_data) = ProbeEntry64::deserialize(data).unwrap();
        data = remaining_data;

        if (entry.start_addr == 0 && entry.end_addr == 0) || entry.start_addr < 20 {
            break;
        }

        entries.push(entry);
    }

    return (entries, data);
}

impl ProbeEntry32 {
    fn deserialize(data: &[u8]) -> Result<(ProbeEntry32, &[u8])> {
        let (partition_type, data) = read_u32_and_advance(data);
        let (partition_name, data) = read_string_and_advance(data, PARTITION_NAME_LEN);
        let (start_addr, data) = read_u32_and_advance(data);
        let (end_addr, data) = read_u32_and_advance(data);

        let pe = ProbeEntry32 {
            partition_type,
            partition_name,
            start_addr,
            end_addr,
        };
        return Ok((pe, data));
    }
}

impl ProbeEntry64 {
    fn deserialize(data: &[u8]) -> Result<(ProbeEntry64, &[u8])> {
        let (partition_type, data) = read_u32_and_advance(data);
        let (partition_name, data) = read_string_and_advance(data, PARTITION_NAME_LEN);
        let (partition_info, data) = read_u64_and_advance(data);
        let (start_addr, data) = read_u64_and_advance(data);
        let (end_addr, data) = read_u64_and_advance(data);

        let pe = ProbeEntry64 {
            partition_type,
            partition_name,
            partition_info,
            start_addr,
            end_addr,
        };
        return Ok((pe, data));
    }
}

// TODO: DRY this into a small parser-combinator module,
// as this code is very similar to the PIT deserializer code.

fn read_u32_and_advance(data: &[u8]) -> (u32, &[u8]) {
    let mut int: [u8; 4] = [0; 4];
    for (i, b) in data[0..3].iter().enumerate() {
        int[i] = *b;
    }

    let data = &data[4..];
    return (u32::from_le_bytes(int), data);
}

fn read_u64_and_advance(data: &[u8]) -> (u64, &[u8]) {
    let mut int: [u8; 8] = [0; 8];
    for (i, b) in data[0..7].iter().enumerate() {
        int[i] = *b;
    }

    let data = &data[8..];
    return (u64::from_le_bytes(int), data);
}

fn read_string_and_advance(data: &[u8], max_len: usize) -> (String, &[u8]) {
    let data = &data[0..max_len];
    // C String constructor fails on seeing a NULL-byte; filter them out
    let str_data: Vec<u8> = data.iter().take_while(|x| **x != 0).map(|x| *x).collect();
    let c_str = CString::new(str_data.clone()).unwrap();
    let c_str_len = c_str.clone().into_bytes_with_nul().len();
    if c_str_len > max_len {
        panic!("String too long!");
    }

    let s = c_str.into_string().unwrap();
    let data = &data[str_data.len()..];
    return (s, data);
}

/// Probes the target for information about it.
///
/// Handshaking and termination must be performed before and after calling this, respectively.
pub fn probe(c: &mut Box<dyn Communicator>) -> Result<ProbeTable> {
    /*
    TODO: Because we don't know how much to read yet,
    we really should just pass in the communicator rather than doing this brittle crap.
    */
    let mut data: Vec<u8> = Vec::new();
    loop {
        let new_data = c.recv_exact(4);
        match new_data {
            Ok(mut new_data) => data.append(&mut new_data),
            Err(_) => break,
        }
    }

    let table = ProbeTable::deserialize(&data);
    return Ok(table);
}
