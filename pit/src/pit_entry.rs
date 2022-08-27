use core::fmt;

#[cfg(feature = "tabled")]
use tabled::Tabled;

#[cfg(feature = "serde")]
use serde::Serialize;

type PitIdentifier = u32;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct PitEntry {
    pub pit_type: PitType,
    pub pit_device_type: PitDeviceType,
    pub pit_id: PitIdentifier,
    #[cfg_attr(feature = "tabled", tabled(display_with = "display_pit_attributes"))]
    pub pit_attributes: Vec<PitAttribute>,
    #[cfg_attr(
        feature = "tabled",
        tabled(display_with = "display_pit_update_attributes")
    )]
    pub pit_update_attributes: Vec<PitUpdateAttribute>,
    pub block_size_or_offset: u32,
    pub block_count: u32,
    pub file_offset: u32,
    pub file_size: u32,
    pub partition_name: String,
    pub flash_filename: String,
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

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[cfg_attr(feature = "serde", derive(Serialize))]
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

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[cfg_attr(feature = "serde", derive(Serialize))]
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
            Emmc => write!(f, "EMMC"),
            Spi => write!(f, "SPI"),
            Ide => write!(f, "IDE"),
            NandX16 => write!(f, "NANDX16"),
            Nor => write!(f, "NOR"),
            NandWB1 => write!(f, "NANDWB1"),
            Ufs => write!(f, "UFS"),
        }
    }
}

impl Into<u32> for PitDeviceType {
    fn into(self) -> u32 {
        use PitDeviceType::*;
        match self {
            Nand => 0x01,
            Emmc => 0x02,
            Spi => 0x03,
            Ide => 0x04,
            NandX16 => 0x05,
            Nor => 0x06,
            NandWB1 => 0x07,
            Ufs => 0x08,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[cfg_attr(feature = "serde", derive(Serialize))]
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

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[cfg_attr(feature = "serde", derive(Serialize))]
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
