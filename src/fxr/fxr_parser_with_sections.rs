use super::{
    Section12Entry, Section13Entry, Section14Entry, parse_section_1_tree::ParsedSections,
    parse_section_4_tree::ParsedSection4Tree,
};
use crate::fxr::{
    Header, parse_section_1_tree::parse_section1_tree, parse_section_4_tree::parse_section4_tree,
    util::parse_named_u32_entries,
};
use log::debug;
use zerocopy::Ref;

pub struct ParsedSection7Nested<'a> {
    pub section11: Vec<Ref<&'a [u8], [crate::fxr::Section11Entry]>>,
    pub section8: Vec<Ref<&'a [u8], [crate::fxr::Section8Entry]>>,
}

pub struct ParsedSection8<'a> {
    pub section11: Vec<Ref<&'a [u8], [crate::fxr::Section11Entry]>>,
    pub section9: Vec<Ref<&'a [u8], [crate::fxr::Section9Entry]>>,
}

pub struct ParsedSection7<'a> {
    pub section11: Vec<Ref<&'a [u8], [crate::fxr::Section11Entry]>>,
    pub section8: Vec<Ref<&'a [u8], [crate::fxr::Section8Entry]>>,
}

pub struct ParsedFXR<'a> {
    pub header: Ref<&'a [u8], Header>,
    pub section1_tree: Option<ParsedSections<'a>>,
    pub section4_tree: Option<ParsedSection4Tree<'a>>,
    pub section12_entries: Ref<&'a [u8], [Section12Entry]>,
    pub section13_entries: Ref<&'a [u8], [Section13Entry]>,
    pub section14_entries: Ref<&'a [u8], [Section14Entry]>,
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
/// fn main() -> anyhow::Result<()> {
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
pub fn parse_fxr<'a>(data: &'a [u8]) -> anyhow::Result<ParsedFXR<'a>> {
    let header_size = std::mem::size_of::<Header>();

    let header_ref = Ref::<_, Header>::from_bytes(&data[..header_size])
        .map_err(|_| anyhow::anyhow!("Failed to read header"))?;

    assert_eq!(&header_ref.magic, b"FXR\0");
    assert_eq!(header_ref.version, 5);
    debug!("Header @ 0x00000000: {:#?}", header_ref);

    let section1_tree = if header_ref.section1_count > 0 {
        Some(parse_section1_tree(data, header_ref.section1_offset)?)
    } else {
        None
    };

    let section4_tree = if header_ref.section4_count > 0 {
        Some(parse_section4_tree(data, header_ref.section4_offset)?)
    } else {
        None
    };

    let section12_entries = parse_named_u32_entries::<Section12Entry>(
        data,
        header_ref.section12_offset,
        header_ref.section12_count,
        "Section12",
    )?;

    let section13_entries = parse_named_u32_entries::<Section13Entry>(
        data,
        header_ref.section13_offset,
        header_ref.section13_count,
        "Section13",
    )?;

    let section14_entries = parse_named_u32_entries::<Section14Entry>(
        data,
        header_ref.section14_offset,
        header_ref.section14_count,
        "Section14",
    )?;

    Ok(ParsedFXR {
        header: header_ref,
        section1_tree,
        section4_tree,
        section12_entries,
        section13_entries,
        section14_entries,
    })
}
