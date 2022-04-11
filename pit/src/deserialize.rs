use std::ffi::CString;

// TODO: DRY
const PIT_ATTRIBUTE_WRITE: u32 = 0x01;
const PIT_ATTRIBUTE_STL: u32 = 0x02;
const PIT_ATTRIBUTE_BML: u32 = 0x04;

const PIT_UPDATE_ATTRIBUTE_FOTA: u32 = 0x01;
const PIT_UPDATE_ATTRIBUTE_SECURE: u32 = 0x02;

const PIT_STRING_MAX_LEN: usize = 32;

use super::*;

impl Pit {
    /// Obtain a PIT structure by parsing it's binary representation.
    pub fn deserialize(data: &[u8]) -> Result<Pit, PitError> {
        // Check whether magic is valid
        if data[0..=3] != PIT_MAGIC {
            return Err(PitError::InvalidPit([data[0], data[1], data[2], data[3]]).into());
        }
        let data = &data[4..];

        // Parse global data
        let (num_entries, data) = read_u32_as_usize_and_advance(data)?;
        let gang_name = read_string_and_advance(&data, 8)?;
        let data = &data[8..];
        let project_name = read_string_and_advance(&data, 8)?;
        let data = &data[8..];
        let (proto_version, mut data) = read_u32_and_advance(&data)?;

        // Parse each entry
        let mut entries: Vec<PitEntry> = Vec::new();
        entries.reserve(num_entries);
        for _ in 0..num_entries {
            let (entry, _) = read_entry(data)?;
            entries.push(entry);
            data = &data[PIT_ENTRY_SIZE..];
        }

        return Ok(Pit {
            gang_name,
            project_name,
            proto_version,
            entries,
            idx: 0,
        });
    }
}

fn read_u32_as_usize_and_advance(data: &[u8]) -> Result<(usize, &[u8]), PitError> {
    let (int, data) = read_u32_and_advance(data)?;
    let int: u32 = int.into();
    let int: usize = int.try_into().unwrap();
    return Ok((int, data));
}

fn read_u32_and_advance(data: &[u8]) -> Result<(u32, &[u8]), PitError> {
    let mut int_raw: [u8; 4] = [0; 4];
    for (i, b) in data[0..3].iter().enumerate() {
        int_raw[i] = *b;
    }

    let int = u32::from_le_bytes(int_raw);
    let data = &data[4..];
    return Ok((int, data));
}

fn read_string_and_advance(data: &[u8], max_len: usize) -> Result<String, PitError> {
    let data = &data[0..max_len];
    // C String constructor fails on seeing a NULL-byte; filter them out
    let data: Vec<u8> = data.iter().take_while(|x| **x != 0).map(|x| *x).collect();
    let c_str = CString::new(data).unwrap();
    let s = c_str.into_string().unwrap();
    return Ok(s);
}

fn read_entry(data: &[u8]) -> Result<(PitEntry, &[u8]), PitError> {
    let (pit_type, data) = read_u32_and_advance(data)?;
    let pit_type = match pit_type {
        0x00 => PitType::Other,
        0x01 => PitType::Modem,
        _ => return Err(PitError::InvalidBinaryType(pit_type).into()),
    };

    let (pit_device_type, data) = read_u32_and_advance(data)?;
    use PitDeviceType::*;
    let pit_device_type = match pit_device_type {
        0x01 => Nand,
        0x02 => Emmc,
        0x03 => Spi,
        0x04 => Ide,
        0x05 => NandX16,
        0x06 => Nor,
        0x07 => NandWB1,
        0x08 => Ufs,
        _ => return Err(PitError::InvalidDeviceType(pit_device_type).into()),
    };

    let (pit_id, data) = read_u32_and_advance(data)?;

    let (pit_attributes_raw, data) = read_u32_and_advance(data)?;
    let pit_attributes_raw: u32 = pit_attributes_raw.into();
    let mut pit_attributes: Vec<PitAttribute> = Vec::new();
    if (pit_attributes_raw & PIT_ATTRIBUTE_WRITE) != 0 {
        pit_attributes.push(PitAttribute::Write);
    }
    if (pit_attributes_raw & PIT_ATTRIBUTE_STL) != 0 {
        pit_attributes.push(PitAttribute::Stl);
    }
    if (pit_attributes_raw & PIT_ATTRIBUTE_BML) != 0 {
        pit_attributes.push(PitAttribute::Bml);
    }

    let (pit_update_attributes_raw, data) = read_u32_and_advance(data)?;
    let pit_update_attributes_raw: u32 = pit_update_attributes_raw.into();
    let mut pit_update_attributes: Vec<PitUpdateAttribute> = Vec::new();
    if (pit_update_attributes_raw & PIT_UPDATE_ATTRIBUTE_FOTA) != 0 {
        pit_update_attributes.push(PitUpdateAttribute::Fota);
    }
    if (pit_update_attributes_raw & PIT_UPDATE_ATTRIBUTE_SECURE) != 0 {
        pit_update_attributes.push(PitUpdateAttribute::Secure);
    }

    let (block_size_or_offset, data) = read_u32_and_advance(data)?;

    let (block_count, data) = read_u32_and_advance(data)?;

    let (file_offset, data) = read_u32_and_advance(data)?;

    let (file_size, data) = read_u32_and_advance(data)?;

    // FIXME: What did we miss to read?
    let (_, data) = read_u32_and_advance(data)?;

    let partition_name = read_string_and_advance(data, PIT_STRING_MAX_LEN)?;
    let data = &data[32..];

    let flash_filename = read_string_and_advance(data, PIT_STRING_MAX_LEN)?;
    let data = &data[32..];

    let fota_filename = read_string_and_advance(data, PIT_STRING_MAX_LEN)?;
    let data = &data[32..];

    return Ok((
        PitEntry {
            pit_type,
            pit_device_type,
            pit_id,
            pit_attributes,
            pit_update_attributes,
            block_size_or_offset,
            block_count,
            file_offset,
            file_size,
            partition_name,
            flash_filename,
            fota_filename,
        },
        data,
    ));
}