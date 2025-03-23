use serde::{Deserialize, Serialize};
use zerocopy_derive::{FromBytes, Immutable, IntoBytes, KnownLayout};

pub mod fxr_parser_with_sections;
pub mod parse_section_1_tree;
pub mod parse_section_3_tree;
pub mod parse_section_4_tree;
pub mod parse_section_6_nested;
pub mod parse_section_7_nested;
pub mod util;
pub mod view;

pub trait U32Field {
    fn data(&self) -> u32;
}

impl U32Field for crate::fxr::Section12Entry {
    fn data(&self) -> u32 {
        self.data
    }
}
impl U32Field for crate::fxr::Section13Entry {
    fn data(&self) -> u32 {
        self.data
    }
}
impl U32Field for crate::fxr::Section14Entry {
    fn data(&self) -> u32 {
        self.data
    }
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout, Serialize)]
pub struct Header {
    pub magic: [u8; 4],
    unk04: u16,
    pub version: u16,
    unk08: u32,
    pub ffx_id: u32,

    pub section1_offset: u32,
    pub section1_count: u32,
    pub section2_offset: u32,
    pub section2_count: u32,
    pub section3_offset: u32,
    pub section3_count: u32,
    pub section4_offset: u32,
    pub section4_count: u32,
    pub section5_offset: u32,
    pub section5_count: u32,
    pub section6_offset: u32,
    pub section6_count: u32,
    pub section7_offset: u32,
    pub section7_count: u32,
    pub section8_offset: u32,
    pub section8_count: u32,
    pub section9_offset: u32,
    pub section9_count: u32,
    pub section10_offset: u32,
    pub section10_count: u32,
    pub section11_offset: u32,
    pub section11_count: u32,
    unk68: u32,
    unk70: u32,

    pub section12_offset: u32,
    pub section12_count: u32,
    pub section13_offset: u32,
    pub section13_count: u32,
    pub section14_offset: u32,
    pub section14_count: u32,
    unk88: u32,
    unk8c: u32,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            magic: [b'F', b'X', b'R', 0],
            unk04: 0,
            version: 1,
            unk08: 0,
            ffx_id: 0,
            section1_offset: 0,
            section1_count: 0,
            section2_offset: 0,
            section2_count: 0,
            section3_offset: 0,
            section3_count: 0,
            section4_offset: 0,
            section4_count: 0,
            section5_offset: 0,
            section5_count: 0,
            section6_offset: 0,
            section6_count: 0,
            section7_offset: 0,
            section7_count: 0,
            section8_offset: 0,
            section8_count: 0,
            section9_offset: 0,
            section9_count: 0,
            section10_offset: 0,
            section10_count: 0,
            section11_offset: 0,
            section11_count: 0,
            unk68: 0,
            unk70: 0,
            section12_offset: 0,
            section12_count: 0,
            section13_offset: 0,
            section13_count: 0,
            section14_offset: 0,
            section14_count: 0,
            unk88: 0,
            unk8c: 0,
        }
    }
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout, Serialize, Deserialize)]
pub struct Section4Container {
    unk00: u16,
    unk02: u8,
    unk03: u8,
    unk04: u32,
    pub section5_count: u32,
    pub section6_count: u32,
    pub section4_count: u32,
    unk14: u32,
    pub section5_offset: u32,
    unk1c: u32,
    pub section6_offset: u32,
    unk24: u32,
    pub section4_offset: u32,
    unk2c: u32,
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct Section4Entry {
    // Placeholder structure
    unk00: u32,
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout)]
pub struct Section5Entry {
    // Placeholder structure
    unk00: u32,
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct Section6Entry {
    unk00: u16,
    unk02: u8,
    unk03: u8,
    unk04: u32,
    pub section11_count1: u32,
    pub section10_count: u32,
    pub section7_count1: u32,
    pub section11_count2: u32,
    unk18: u32,
    pub section7_count2: u32,
    pub section11_offset: u32,
    unk24: u32,
    pub section10_offset: u32,
    unk2c: u32,
    pub section7_offset: u32,
    unk34: u32,
    unk38: u32,
    unk3c: u32,
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout, Serialize, Deserialize)]
pub struct Section1Container {
    unk00: u32,
    pub section2_count: u32,
    pub section2_offset: u32,
    unk0c: u32,
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout, Serialize, Deserialize)]
pub struct Section2Container {
    unk00: u32,
    pub section3_count: u32,
    pub section3_offset: u32,
    unk0c: u32,
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, Immutable, KnownLayout, Serialize, Deserialize)]
pub struct Section3Entry {
    unk00: u16,
    unk01: u8,
    unk02: u8,
    unk04: u32,
    unk08: u32,
    unk0c: u32,
    unk10: u32,
    unk14: u32,
    unk18: u32,
    unk1c: u32,
    pub section11_offset1: u32,
    unk24: u32,
    unk28: u32,
    unk2c: u32,
    unk30: u32,
    unk34: u32,
    unk38: u32,
    unk3c: u32,
    unk40: u32,
    unk44: u32,
    pub section11_offset2: u32,
    unk4c: u32,
    unk50: u32,
    unk54: u32,
    unk58: u32,
    unk5c: u32,
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct Section7Container {
    unk00: u32,
    unk04: u32,
    pub section11_count: u32,
    unk0c: u32,
    pub section11_offset: u32,
    unk14: u32,
    pub section8_offset: u32,
    unk1c: u32,
    pub section8_count: u32,
    unk24: u32,
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct Section8Container {
    unk00: u8,
    unk01: u8,
    unk02: u8,
    unk03: u8,
    unk04: u32,
    pub section11_count: u32,
    pub section9_count: u32,
    pub section11_offset: u32,
    unk14: u32,
    pub section9_offset: u32,
    unk1c: u32,
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct Section9Container {
    unk00: u32,
    unk04: u32,
    pub section11_count: u32,
    unk0c: u32,
    pub section11_offset: u32,
    unk14: u32,
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct Section10Container {
    pub section11_offset: u32,
    unk04: u32,
    pub section11_count: u32,
    unk0c: u32,
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct Section12Entry {
    data: u32, // Assuming each entry is 4 bytes
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct Section13Entry {
    data: u32,
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct Section11Entry {
    pub data: u32,
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct Section14Entry {
    data: u32,
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct Section7Entry {
    unk00: u32,
    unk04: u32,
    pub section11_count: u32,
    unk0c: u32,
    pub section11_offset: u32,
    unk14: u32,
    pub section8_offset: u32,
    unk1c: u32,
    pub section8_count: u32,
    unk24: u32,
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct Section10Entry {
    pub section11_offset: u32,
    unk04: u32,
    pub section11_count: u32,
    unk0c: u32,
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct Section8Entry {
    unk00: u8,
    unk01: u8,
    unk02: u8,
    unk03: u8,
    unk04: u32,
    pub section11_count: u32,
    pub section9_count: u32,
    pub section11_offset: u32,
    unk14: u32,
    pub section9_offset: u32,
    unk1c: u32,
}

#[repr(C)]
#[derive(Debug, FromBytes, IntoBytes, KnownLayout, Immutable)]
pub struct Section9Entry {
    unk00: u32,
    unk04: u32,
    pub section11_count: u32,
    unk0c: u32,
    pub section11_offset: u32,
    unk14: u32,
}
