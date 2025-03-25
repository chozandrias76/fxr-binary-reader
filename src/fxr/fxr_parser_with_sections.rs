use super::{
    Section12Entry, Section13Entry, Section14Entry,
    parse_section_1_tree::ParsedSections,
    parse_section_4_tree::ParsedSection4Tree,
    util::{ParseError, parse_section_slice},
};
use crate::fxr::{
    Header, parse_section_1_tree::parse_section1_tree, parse_section_4_tree::parse_section4_tree,
};
use std::error::Error;
use validator::Validate;
use zerocopy::Ref;

pub struct ParsedFXR<'a> {
    pub header: Ref<&'a [u8], Header>,
    pub section1_tree: Option<ParsedSections<'a>>,
    pub section4_tree: Option<ParsedSection4Tree<'a>>,
    pub section12_entries: Option<Ref<&'a [u8], [Section12Entry]>>,
    pub section13_entries: Option<Ref<&'a [u8], [Section13Entry]>>,
    pub section14_entries: Option<Ref<&'a [u8], [Section14Entry]>>,
}

impl Validate for ParsedFXR<'_> {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        self.header.validate()?;
        if let Some(ref section1_tree) = self.section1_tree {
            section1_tree.validate()?;
        }
        if let Some(ref section4_tree) = self.section4_tree {
            section4_tree.validate()?;
        }
        if let Some(ref entries) = self.section12_entries {
            for entry in entries.iter() {
                entry.validate()?;
            }
        }
        if let Some(ref entries) = self.section13_entries {
            for entry in entries.iter() {
                entry.validate()?;
            }
        }
        if let Some(ref entries) = self.section14_entries {
            for entry in entries.iter() {
                entry.validate()?;
            }
        }
        Ok(())
    }
}

/// Parses the FXR file and prints the header and sections information.
/// # Example
/// ```rust
/// use fxr_binary_reader::fxr::fxr_parser_with_sections::parse_fxr;
/// use memmap2::Mmap;
/// use std::fs::File;
/// use std::path::PathBuf;
/// use zerocopy::IntoBytes;
/// use log::error;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let path = PathBuf::from("./fixtures/f000302421.fxr");
///     let file = File::open(path)?;
///     let mmap = unsafe { Mmap::map(&file)? };
///     let data = &mmap.as_bytes();
///     if let Err(e) = parse_fxr(data) {
///         error!("Error parsing FXR: {}", e);
///         panic!("Test failed");
///     }
///     Ok(())
/// }
/// ```
pub fn parse_fxr<'a>(fxr_file_bytes: &'a [u8]) -> Result<ParsedFXR<'a>, Box<dyn Error>> {
    let header_size = std::mem::size_of::<Header>();

    let header_ref =
        Ref::<_, Header>::from_bytes(&fxr_file_bytes[..header_size]).map_err(|_| {
            ParseError::InvalidHeader(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid Header",
            )))
        })?;
    header_ref.validate()?;

    let section1_tree = if header_ref.section1_count > 0 {
        Some(parse_section1_tree(
            fxr_file_bytes,
            header_ref.section1_offset,
        )?)
    } else {
        None
    };

    let section4_tree = if header_ref.section4_count > 0 {
        Some(parse_section4_tree(
            fxr_file_bytes,
            header_ref.section4_offset,
        )?)
    } else {
        None
    };

    let section12_entries = if header_ref.section12_count > 0 {
        Some(parse_section_slice::<Section12Entry>(
            fxr_file_bytes,
            header_ref.section12_offset,
            header_ref.section12_count,
            "Section12",
        )?)
    } else {
        None
    };

    let section13_entries = if header_ref.section13_count > 0 {
        Some(parse_section_slice::<Section13Entry>(
            fxr_file_bytes,
            header_ref.section13_offset,
            header_ref.section13_count,
            "Section13",
        )?)
    } else {
        None
    };

    let section14_entries = if header_ref.section14_count > 0 {
        Some(parse_section_slice::<Section14Entry>(
            fxr_file_bytes,
            header_ref.section14_offset,
            header_ref.section14_count,
            "Section14",
        )?)
    } else {
        None
    };

    Ok(ParsedFXR {
        header: header_ref,
        section1_tree,
        section4_tree,
        section12_entries,
        section13_entries,
        section14_entries,
    })
}
