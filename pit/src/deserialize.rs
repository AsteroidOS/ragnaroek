use std::ffi::CString;

use super::*;

// TODO: DRY
const PIT_ATTRIBUTE_WRITE: u32 = 0x01;
const PIT_ATTRIBUTE_STL: u32 = 0x02;
const PIT_ATTRIBUTE_BML: u32 = 0x04;

const PIT_UPDATE_ATTRIBUTE_FOTA: u32 = 0x01;
const PIT_UPDATE_ATTRIBUTE_SECURE: u32 = 0x02;

const PIT_STRING_MAX_LEN: usize = 32;

fn is_pit_v2(data: &[u8]) -> bool {
    // According to Samsung-Loki, the way to detect version is to check whether all block sizes are the same.
    let data = &data[PIT_HEADER_SIZE..];
    let mut last_block_size: Option<u32> = None;
    for entry in data.chunks(PIT_ENTRY_SIZE) {
        // Compare block size to the last one we encountered
        let block_size = u32::from_be_bytes([entry[20], entry[21], entry[22], entry[23]]);
        if Some(block_size) != last_block_size {
            if last_block_size.is_none() {
                last_block_size = Some(block_size);
            }
            if last_block_size.unwrap() != block_size {
                return true;
            }
            last_block_size = Some(block_size);
        }
    }
    return false;
}

impl Pit {
    /// Obtain a PIT structure by parsing it's binary representation.
    pub fn deserialize(data: &[u8]) -> Result<Pit, PitError> {
        // Check whether magic is valid
        if data[0..=3] != PIT_MAGIC {
            return Err(PitError::InvalidPit([data[0], data[1], data[2], data[3]]).into());
        }
        let is_v2 = is_pit_v2(data);
        let data = &data[4..];

        // Parse global data
        let (num_entries, data) = read_u32_as_usize_and_advance(data)?;
        let gang_name = read_string_and_advance(&data, 8)?;
        let data = &data[8..];
        let project_name = read_string_and_advance(&data, 8)?;
        let data = &data[8..];
        // The purpose of this value is not known
        let (_, mut data) = read_u32_and_advance(&data)?;

        // Parse each entry
        if is_v2 {
            let mut entries: Vec<PitEntryV2> = Vec::new();
            entries.reserve(num_entries);
            for _ in 0..num_entries {
                let (entry, _) = read_entry_v2(data)?;
                entries.push(entry);
                data = &data[PIT_ENTRY_SIZE..];
            }

            return Ok(Pit::from_v2(PitV2 {
                gang_name,
                project_name,
                entries,
                idx: 0,
            }));
        } else {
            let mut entries: Vec<PitEntryV1> = Vec::new();
            entries.reserve(num_entries);
            for _ in 0..num_entries {
                let (entry, _) = read_entry_v1(data)?;
                entries.push(entry);
                data = &data[PIT_ENTRY_SIZE..];
            }

            return Ok(Pit::from_v1(PitV1 {
                gang_name,
                project_name,
                entries,
                idx: 0,
            }));
        }
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

fn read_pit_type_and_advance(data: &[u8]) -> Result<(PitType, &[u8]), PitError> {
    let (pit_type, data) = read_u32_and_advance(data)?;
    let pit_type = match pit_type {
        0x00 => PitType::Other,
        0x01 => PitType::Modem,
        _ => return Err(PitError::InvalidBinaryType(pit_type).into()),
    };
    return Ok((pit_type, data));
}

fn read_pit_device_type_and_advance(data: &[u8]) -> Result<(PitDeviceType, &[u8]), PitError> {
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
    return Ok((pit_device_type, data));
}

fn read_pit_attrs_and_advance(data: &[u8]) -> Result<(Vec<PitAttribute>, &[u8]), PitError> {
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
    return Ok((pit_attributes, data));
}

fn read_pit_update_attrs_and_advance(
    data: &[u8],
) -> Result<(Vec<PitUpdateAttribute>, &[u8]), PitError> {
    let (pit_update_attributes_raw, data) = read_u32_and_advance(data)?;
    let pit_update_attributes_raw: u32 = pit_update_attributes_raw.into();
    let mut pit_update_attributes: Vec<PitUpdateAttribute> = Vec::new();
    if (pit_update_attributes_raw & PIT_UPDATE_ATTRIBUTE_FOTA) != 0 {
        pit_update_attributes.push(PitUpdateAttribute::Fota);
    }
    if (pit_update_attributes_raw & PIT_UPDATE_ATTRIBUTE_SECURE) != 0 {
        pit_update_attributes.push(PitUpdateAttribute::Secure);
    }
    return Ok((pit_update_attributes, data));
}

fn read_pit_partition_type_and_advance(data: &[u8]) -> Result<(u32, &[u8]), PitError> {
    return read_u32_and_advance(data);
}

fn read_fs_type_and_advance(data: &[u8]) -> Result<(u32, &[u8]), PitError> {
    return read_u32_and_advance(data);
}

fn read_entry_v1(data: &[u8]) -> Result<(PitEntryV1, &[u8]), PitError> {
    let (pit_type, data) = read_pit_type_and_advance(data)?;
    let (pit_device_type, data) = read_pit_device_type_and_advance(data)?;
    let (partition_id, data) = read_u32_and_advance(data)?;
    let (pit_attributes, data) = read_pit_attrs_and_advance(data)?;
    let (pit_update_attributes, data) = read_pit_update_attrs_and_advance(data)?;
    let (block_size, data) = read_u32_and_advance(data)?;
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
        PitEntryV1 {
            pit_type,
            pit_device_type,
            partition_id,
            pit_attributes,
            pit_update_attributes,
            block_size,
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

fn read_entry_v2(data: &[u8]) -> Result<(PitEntryV2, &[u8]), PitError> {
    let (pit_type, data) = read_pit_type_and_advance(data)?;
    let (pit_device_type, data) = read_pit_device_type_and_advance(data)?;
    let (partition_id, data) = read_u32_and_advance(data)?;
    let (partition_type, data) = read_pit_partition_type_and_advance(data)?;
    let (pit_filesystem, data) = read_fs_type_and_advance(data)?;
    let (start_block, data) = read_u32_and_advance(data)?;
    let (block_num, data) = read_u32_and_advance(data)?;
    let (file_offset, data) = read_u32_and_advance(data)?;
    let (file_size, data) = read_u32_and_advance(data)?;
    let partition_name = read_string_and_advance(data, PIT_STRING_MAX_LEN)?;
    let data = &data[32..];
    let flash_filename = read_string_and_advance(data, PIT_STRING_MAX_LEN)?;
    let data = &data[32..];
    let fota_filename = read_string_and_advance(data, PIT_STRING_MAX_LEN)?;
    let data = &data[32..];

    return Ok((
        PitEntryV2 {
            pit_type,
            pit_device_type,
            partition_id,
            partition_type,
            pit_filesystem,
            start_block,
            block_num,
            file_offset,
            file_size,
            partition_name,
            flash_filename,
            fota_filename,
        },
        data,
    ));
}
