use core::fmt;

#[cfg(feature = "tabled")]
use tabled::Tabled;

#[cfg(feature = "serde")]
use serde::Serialize;

type PitIdentifier = u32;
// TODO: Should be an enum, find values
type PitPartitionType = u32;
// TODO: Should be an enum, find values
type PitFilesystem = u32;

/// PIT entry for a version 1 PIT file, describing a partition.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct PitEntryV1 {
    /// The type of PIT.
    pub pit_type: PitType,
    /// The type of device (e.g. eMMC/NAND etc.)
    pub pit_device_type: PitDeviceType,
    /// Identifier for this partition.
    pub partition_id: PitIdentifier,
    /// Attributes of this entry.
    #[cfg_attr(feature = "tabled", tabled(display_with = "display_pit_attributes"))]
    pub pit_attributes: Vec<PitAttribute>,
    #[cfg_attr(
        feature = "tabled",
        tabled(display_with = "display_pit_update_attributes")
    )]
    /// Attributes relevant for updating the partition.
    pub pit_update_attributes: Vec<PitUpdateAttribute>,
    /// Size of one block on the device.
    pub block_size: u32,
    /// Number of blocks.
    pub block_count: u32,
    /// TODO: Document
    pub file_offset: u32,
    /// TODO: Document
    pub file_size: u32,
    /// Name of the partition. Must be specified to flash it.
    pub partition_name: String,
    /// Name of the file customarily used to flash this partition by Odin.
    pub flash_filename: String,
    /// Name of the file used to update this partition from an OTA bundle.
    pub fota_filename: String,
}

/// PIT entry for a version 1 PIT file, describing a partition.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct PitEntryV2 {
    /// TODO: Document
    pub pit_type: PitType,
    /// TODO: Document
    pub pit_device_type: PitDeviceType,
    /// TODO: Document
    pub partition_id: PitIdentifier,
    /// TODO: Document
    pub partition_type: PitPartitionType,
    /// TODO: Document
    pub pit_filesystem: PitFilesystem,
    /// TODO: Document
    pub start_block: u32,
    /// TODO: Document
    pub block_num: u32,
    /// TODO: Document
    pub file_offset: u32,
    /// TODO: Document
    pub file_size: u32,
    /// TODO: Document
    pub partition_name: String,
    /// TODO: Document
    pub flash_filename: String,
    /// TODO: Document
    pub fota_filename: String,
}

#[cfg(feature = "tabled")]
fn display_pit_attributes(attrs: &Vec<PitAttribute>) -> String {
    let mut s = String::new();
    for a in attrs {
        s.push_str(&format!("{}\n", a));
    }
    return s;
}

#[cfg(feature = "tabled")]
fn display_pit_update_attributes(attrs: &Vec<PitUpdateAttribute>) -> String {
    let mut s = String::new();
    for a in attrs {
        s.push_str(&format!("{}\n", a));
    }
    return s;
}

/// The component this partition belongs to.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum PitType {
    /// A partition for some other component (e.g. the AP)
    Other = 0x00,
    /// A modem partition. Not always set in practice for newer devices.
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

/// Type of storage device backing this partition.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum PitDeviceType {
    /// TODO: Document
    OneNand = 0x00,
    /// TODO: Document
    Nand = 0x01,
    /// TODO: Document
    EmmcOrMoviNand = 0x02,
    /// TODO: Document
    Spi = 0x03,
    /// TODO: Document
    Ide = 0x04,
    /// TODO: Document
    NandX16 = 0x05,
    /// TODO: Document
    Nor = 0x06,
    /// TODO: Document
    NandWB1 = 0x07,
    /// TODO: Document
    Ufs = 0x08,
}

impl fmt::Display for PitDeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use PitDeviceType::*;
        match self {
            OneNand => write!(f, "ONENAND"),
            Nand => write!(f, "NAND"),
            EmmcOrMoviNand => write!(f, "EMMC/MOVINAND"),
            Spi => write!(f, "SPI"),
            Ide => write!(f, "IDE"),
            NandX16 => write!(f, "NANDX16"),
            Nor => write!(f, "NOR"),
            NandWB1 => write!(f, "NANDWB1"),
            Ufs => write!(f, "UFS"),
        }
    }
}

impl From<PitDeviceType> for u32 {
    fn from(val: PitDeviceType) -> u32 {
        use PitDeviceType::*;
        match val {
            OneNand => 0x00,
            Nand => 0x01,
            EmmcOrMoviNand => 0x02,
            Spi => 0x03,
            Ide => 0x04,
            NandX16 => 0x05,
            Nor => 0x06,
            NandWB1 => 0x07,
            Ufs => 0x08,
        }
    }
}

impl TryFrom<u32> for PitDeviceType {
    type Error = ();
    fn try_from(val: u32) -> Result<PitDeviceType, Self::Error> {
        use PitDeviceType::*;
        match val {
            0x00 => Ok(OneNand),
            0x01 => Ok(Nand),
            0x02 => Ok(EmmcOrMoviNand),
            0x03 => Ok(Spi),
            0x04 => Ok(Ide),
            0x05 => Ok(NandX16),
            0x06 => Ok(Nor),
            0x07 => Ok(NandWB1),
            0x08 => Ok(Ufs),
            _ => Err(()),
        }
    }
}

/// Misc. attributes for the partition.
/// I don't quite know what they mean.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum PitAttribute {
    /// TODO: Document
    Write = 0x01,
    /// TODO: Document
    Stl = 0x02,
    /// TODO: Document
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

/// Flags describing how this partition should be updated.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum PitUpdateAttribute {
    /// TODO: Document
    Fota = 0x01,
    /// TODO: Document
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
