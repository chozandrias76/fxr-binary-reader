use serde::{Deserialize, Serialize};
use std::fmt::Display;
use thiserror::Error;
use validator::{Validate, ValidationError};
use zerocopy::IntoBytes;
use zerocopy_derive::{FromBytes, Immutable, IntoBytes, KnownLayout};

pub mod fxr_parser_with_sections;
pub mod parse_section_1_tree;
pub mod parse_section_4_tree;
pub mod parse_section_6_nested;
pub mod util;

mod hex_formatted_bytes {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: std::fmt::LowerHex + std::fmt::UpperHex,
    {
        serializer.serialize_str(&format!("0x{:X}", value))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        u32::from_str_radix(s.trim_start_matches("0x"), 16).map_err(serde::de::Error::custom)
    }
}
mod string_formatted_bytes {
    use serde::{self, Serializer};
    use std::str;

    // Serialize a u32 as a string from its raw bytes
    pub fn serialize<S>(value: &u32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = value.to_le_bytes(); // Convert u32 to little-endian bytes
        let string = str::from_utf8(&bytes).expect("Invalid UTF-8 sequence");
        serializer.serialize_str(string)
    }
}

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
fn validate_fxr_type_magic_bytes(magic: u32) -> Result<(), ValidationError> {
    let mut err = ValidationError::new("Header Magic");
    if magic != u32::from_le_bytes([b'F', b'X', b'R', 0]) {
        err.message = Some(
            format!(
                "Invalid magic value. Found: {}",
                String::from_utf8_lossy(magic.as_bytes())
            )
            .into(),
        );
        Err(err)
    } else {
        Ok(())
    }
}

#[repr(C)]
#[derive(Error, Debug, FromBytes, IntoBytes, Immutable, KnownLayout, Serialize, Validate)]
#[validate(schema(function = "validate_conditional_fields", skip_on_field_errors = false))]
pub struct Header {
    #[validate(custom(function = "validate_fxr_type_magic_bytes"))]
    #[serde(with = "string_formatted_bytes")]
    pub magic: u32,
    unk04: u16,
    #[validate(range(min = 4, max = 5))]
    pub version: u16,
    #[validate(range(min = 1, max = 1))]
    unk08: u32,
    pub ffx_id: u32,
    #[serde(with = "hex_formatted_bytes")]
    pub section1_offset: u32,
    #[validate(range(min = 1, max = 1))]
    pub section1_count: u32,
    #[serde(with = "hex_formatted_bytes")]
    pub section2_offset: u32,
    pub section2_count: u32,
    #[serde(with = "hex_formatted_bytes")]
    pub section3_offset: u32,
    pub section3_count: u32,
    #[serde(with = "hex_formatted_bytes")]
    pub section4_offset: u32,
    pub section4_count: u32,
    #[serde(with = "hex_formatted_bytes")]
    pub section5_offset: u32,
    pub section5_count: u32,
    #[serde(with = "hex_formatted_bytes")]
    pub section6_offset: u32,
    pub section6_count: u32,
    #[serde(with = "hex_formatted_bytes")]
    pub section7_offset: u32,
    pub section7_count: u32,
    #[serde(with = "hex_formatted_bytes")]
    pub section8_offset: u32,
    pub section8_count: u32,
    #[serde(with = "hex_formatted_bytes")]
    pub section9_offset: u32,
    pub section9_count: u32,
    #[serde(with = "hex_formatted_bytes")]
    pub section10_offset: u32,
    pub section10_count: u32,
    #[serde(with = "hex_formatted_bytes")]
    pub section11_offset: u32,
    pub section11_count: u32,
    #[validate(range(min = 1, max = 1))]
    unk68: u32,
    #[validate(range(min = 0, max = 0))]
    unk70: u32,

    #[serde(with = "hex_formatted_bytes")]
    pub section12_offset: u32,
    pub section12_count: u32,
    #[serde(with = "hex_formatted_bytes")]
    pub section13_offset: u32,
    pub section13_count: u32,
    #[serde(with = "hex_formatted_bytes")]
    pub section14_offset: u32,
    pub section14_count: u32,
    unk88: u32,
    unk8c: u32,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            magic: u32::from_le_bytes([b'F', b'X', b'R', 0]),
            unk04: 0,
            version: 1,
            unk08: 0,
            ffx_id: 0,
            section1_offset: 0,
            section1_count: 1,
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

fn validate_conditional_fields(header: &&Header) -> Result<(), ValidationError> {
    let mut error_messages: Vec<String> = vec![];
    let mut err = ValidationError::new("version_5_conditional_header_values");

    if header.version == 5 {
        if header.section12_count > 2 {
            error_messages.push("Section12 count must be less than or equal to 2".to_string());
        }
        if header.section13_count > 2 {
            error_messages.push("Section13 count must be less than or equal to 2".to_string());
        }
        if header.section14_count != 0 {
            error_messages.push("Section14 count must be 0".to_string());
        }
        if header.unk88 != 0 {
            error_messages.push("unk88 must be 0".to_string());
        }
        if header.unk8c != 0 {
            error_messages.push("unk8c must be 0".to_string());
        }
    }
    if !error_messages.is_empty() {
        err.message = Some(error_messages.join(", ").into());
        return Err(err);
    }

    // Add more conditional validations as needed
    Ok(())
}

#[repr(C)]
#[derive(
    Error, Validate, Debug, FromBytes, IntoBytes, Immutable, KnownLayout, Serialize, Deserialize,
)]
pub struct Section4Container {
    unk00: u16,
    unk02: u8,
    unk03: u8,
    #[validate(range(min = 0, max = 0))]
    unk04: u32,
    pub section5_count: u32,
    pub section6_count: u32,
    pub section4_count: u32,
    #[validate(range(min = 0, max = 0))]
    unk14: u32,
    pub section5_offset: u32,
    #[validate(range(min = 0, max = 0))]
    unk1c: u32,
    pub section6_offset: u32,
    #[validate(range(min = 0, max = 0))]
    unk24: u32,
    pub section4_offset: u32,
    #[validate(range(min = 0, max = 0))]
    unk2c: u32,
}

#[repr(C)]
#[derive(
    Error, Validate, Debug, FromBytes, IntoBytes, Immutable, KnownLayout, Serialize, Deserialize,
)]
pub struct Section4Entry {
    // Placeholder structure
    unk00: u32,
}

#[repr(C)]
#[derive(
    Error, Validate, Debug, FromBytes, IntoBytes, Immutable, KnownLayout, Serialize, Deserialize,
)]
pub struct Section5Entry {
    // Placeholder structure
    unk00: u32,
}

#[repr(C)]
#[derive(
    Error, Validate, Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Serialize, Deserialize,
)]
pub struct Section6Entry {
    unk00: u16,
    unk02: u8,
    unk03: u8,
    unk04: u32,
    pub section11_count1: u32,
    pub section10_count: u32,
    pub section7_count1: u32,
    pub section11_count2: u32,
    #[validate(range(min = 0, max = 0))]
    unk18: u32,
    pub section7_count2: u32,
    #[serde(with = "hex_formatted_bytes")]
    pub section11_offset: u32,
    #[validate(range(min = 0, max = 0))]
    unk24: u32,
    #[serde(with = "hex_formatted_bytes")]
    pub section10_offset: u32,
    #[validate(range(min = 0, max = 0))]
    unk2c: u32,
    #[serde(with = "hex_formatted_bytes")]
    pub section7_offset: u32,
    #[validate(range(min = 0, max = 0))]
    unk34: u32,
    #[validate(range(min = 0, max = 0))]
    unk38: u32,
    #[validate(range(min = 0, max = 0))]
    unk3c: u32,
}

#[repr(C)]
#[derive(
    Error, Debug, FromBytes, IntoBytes, Immutable, KnownLayout, Serialize, Deserialize, Validate,
)]
pub struct Section1Container {
    #[validate(range(min = 0, max = 0))]
    unk00: u32,
    pub section2_count: u32,
    pub section2_offset: u32,
    #[validate(range(min = 0, max = 0))]
    unk0c: u32,
}

#[repr(C)]
#[derive(
    Error, Debug, FromBytes, IntoBytes, Immutable, KnownLayout, Serialize, Deserialize, Validate,
)]
pub struct Section2Container {
    #[validate(range(min = 0, max = 0))]
    unk00: u32,
    pub section3_count: u32,
    #[serde(with = "hex_formatted_bytes")]
    pub section3_offset: u32,
    #[validate(range(min = 0, max = 0))]
    unk0c: u32,
}

#[repr(C)]
#[derive(
    Error, Debug, FromBytes, IntoBytes, Immutable, KnownLayout, Serialize, Deserialize, Validate,
)]
pub struct Section3Entry {
    #[validate(range(min = 11, max = 10))]
    unk00: u16,
    #[validate(range(min = 0, max = 0))]
    unk01: u8,
    #[validate(range(min = 1, max = 1))]
    unk02: u8,
    #[validate(range(min = 0, max = 0))]
    unk04: u32,
    unk08: u32,
    #[validate(range(min = 0, max = 0))]
    unk0c: u32,
    unk10: u32,
    #[validate(range(min = 0, max = 0))]
    unk14: u32,
    #[validate(range(min = 1, max = 1))]
    unk18: u32,
    #[validate(range(min = 0, max = 0))]
    unk1c: u32,
    pub section11_offset1: u32,
    #[validate(range(min = 0, max = 0))]
    unk24: u32,
    #[validate(range(min = 0, max = 0))]
    unk28: u32,
    #[validate(range(min = 0, max = 0))]
    unk2c: u32,
    #[validate(range(min = 0, max = 0))]
    unk30: u32,
    #[validate(range(min = 0, max = 0))]
    unk34: u32,
    #[validate(range(min = 0x100FFFC, max = 0x100FFFD))]
    unk38: u32,
    #[validate(range(min = 0, max = 0))]
    unk3c: u32,
    #[validate(range(min = 0, max = 1))]
    unk40: u32,
    #[validate(range(min = 0, max = 0))]
    unk44: u32,
    pub section11_offset2: u32,
    #[validate(range(min = 0, max = 0))]
    unk4c: u32,
    #[validate(range(min = 0, max = 0))]
    unk50: u32,
    #[validate(range(min = 0, max = 0))]
    unk54: u32,
    #[validate(range(min = 0, max = 0))]
    unk58: u32,
    #[validate(range(min = 0, max = 0))]
    unk5c: u32,
}

#[repr(C)]
#[derive(
    Error,
    Validate,
    Debug,
    FromBytes,
    IntoBytes,
    KnownLayout,
    Immutable,
    Serialize,
    Deserialize,
    Default,
)]
pub struct Section7Container {
    #[serde(with = "hex_formatted_bytes")]
    unk00: u32,
    unk04: u32,
    #[serde(with = "hex_formatted_bytes")]
    pub section11_count: u32,
    #[validate(range(min = 0, max = 0))]
    unk0c: u32,
    #[serde(with = "hex_formatted_bytes")]
    pub section11_offset: u32,
    #[validate(range(min = 0, max = 0))]
    unk14: u32,
    #[serde(with = "hex_formatted_bytes")]
    pub section8_offset: u32,
    #[validate(range(min = 0, max = 0))]
    unk1c: u32,
    #[serde(with = "hex_formatted_bytes")]
    pub section8_count: u32,
    #[validate(range(min = 0, max = 0))]
    unk24: u32,
}

#[repr(C)]
#[derive(
    Error, Validate, Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Serialize, Deserialize,
)]
pub struct Section8Container {
    unk00: u8,
    unk01: u8,
    unk02: u8,
    unk03: u8,
    unk04: u32,
    pub section11_count: u32,
    pub section9_count: u32,
    #[serde(with = "hex_formatted_bytes")]
    pub section11_offset: u32,
    #[validate(range(min = 0, max = 0))]
    unk14: u32,
    #[serde(with = "hex_formatted_bytes")]
    pub section9_offset: u32,
    #[validate(range(min = 0, max = 0))]
    unk1c: u32,
}

#[repr(C)]
#[derive(
    Error, Validate, Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Serialize, Deserialize,
)]
pub struct Section9Container {
    unk00: u32,
    unk04: u32,
    pub section11_count: u32,
    #[validate(range(min = 0, max = 0))]
    unk0c: u32,
    #[serde(with = "hex_formatted_bytes")]
    pub section11_offset: u32,
    #[validate(range(min = 0, max = 0))]
    unk14: u32,
}

#[repr(C)]
#[derive(
    Error, Validate, Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Serialize, Deserialize,
)]
pub struct Section10Container {
    #[serde(with = "hex_formatted_bytes")]
    pub section11_offset: u32,
    #[validate(range(min = 0, max = 0))]
    unk04: u32,
    pub section11_count: u32,
    #[validate(range(min = 0, max = 0))]
    unk0c: u32,
}

#[repr(C)]
#[derive(
    Error, Validate, Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Serialize, Deserialize,
)]
pub struct Section12Entry {
    data: u32, // Assuming each entry is 4 bytes
}

#[repr(C)]
#[derive(
    Error, Validate, Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Serialize, Deserialize,
)]
pub struct Section13Entry {
    data: u32,
}

#[repr(C)]
#[derive(
    Error, Validate, Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Serialize, Deserialize,
)]
pub struct Section11Entry {
    pub data: u32,
}

#[repr(C)]
#[derive(
    Error, Validate, Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Serialize, Deserialize,
)]
pub struct Section14Entry {
    data: u32,
}

#[repr(C)]
#[derive(Error, Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Serialize, Deserialize)]
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
#[derive(Error, Debug, FromBytes, IntoBytes, KnownLayout, Immutable, Serialize, Deserialize)]
pub struct Section9Entry {
    unk00: u32,
    unk04: u32,
    pub section11_count: u32,
    unk0c: u32,
    pub section11_offset: u32,
    unk14: u32,
}

impl Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Header {{ magic: 0x{:X}, version: {}, ffx_id: {}, ... }}",
            self.magic, self.version, self.ffx_id
        )
    }
}

impl Display for Section1Container {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Section1Container {{ section2_count: {}, section2_offset: 0x{:X} }}",
            self.section2_count, self.section2_offset
        )
    }
}

impl Display for Section2Container {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Section2Container {{ section3_count: {}, section3_offset: 0x{:X} }}",
            self.section3_count, self.section3_offset
        )
    }
}

impl Display for Section3Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Section3Entry {{ section11_offset1: 0x{:X}, section11_offset2: 0x{:X} }}",
            self.section11_offset1, self.section11_offset2
        )
    }
}

impl Display for Section4Container {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Section4Container {{ section4_count: {}, section5_offset: 0x{:X}, section6_offset: 0x{:X} }}",
            self.section4_count, self.section5_offset, self.section6_offset
        )
    }
}

impl Display for Section4Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Section4Entry {{ unk00: 0x{:X} }}", self.unk00)
    }
}

impl Display for Section5Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Section5Entry {{ unk00: 0x{:X} }}", self.unk00)
    }
}

impl Display for Section6Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Section6Entry {{ section11_offset: 0x{:X}, section10_offset: 0x{:X}, section7_offset: 0x{:X} }}",
            self.section11_offset, self.section10_offset, self.section7_offset
        )
    }
}

impl Display for Section7Container {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Section7Container {{ section11_offset: 0x{:X}, section8_offset: 0x{:X} }}",
            self.section11_offset, self.section8_offset
        )
    }
}

impl Display for Section8Container {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Section8Container {{ section11_offset: 0x{:X}, section9_offset: 0x{:X} }}",
            self.section11_offset, self.section9_offset
        )
    }
}

impl Display for Section9Container {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Section9Container {{ section11_offset: 0x{:X} }}",
            self.section11_offset
        )
    }
}

impl Display for Section10Container {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Section10Container {{ section11_offset: 0x{:X}, section11_count: {} }}",
            self.section11_offset, self.section11_count
        )
    }
}

impl std::fmt::Display for Section11Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Section11Entry {{ data: {} }}", self.data)
    }
}

impl Display for Section12Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Section12Entry {{ data: 0x{:X} }}", self.data)
    }
}

impl Display for Section13Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Section13Entry {{ data: 0x{:X} }}", self.data)
    }
}

impl Display for Section14Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Section14Entry {{ data: 0x{:X} }}", self.data)
    }
}

impl Display for Section8Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Section8Entry {{ section11_count: {}, section9_count: {} }}",
            self.section11_count, self.section9_count
        )
    }
}

impl Display for Section9Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Section9Entry {{ section11_count: {}, section11_offset: 0x{:X} }}",
            self.section11_count, self.section11_offset
        )
    }
}
