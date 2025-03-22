use zerocopy::Ref;

use crate::fxr::parse_section_3_tree::parse_section3_tree;
use crate::fxr::util::parse_struct;
use crate::fxr::{Section1Container, Section2Container};

use super::Section3Entry;
use log::debug;

/// Parses the Section1 tree structure from the given binary data.
///
/// This function reads and processes a hierarchical structure starting with
/// Section1, followed by Section2, and optionally Section3 if present. It uses
/// helper functions to parse each section and prints debug information about
/// the parsed structures.
///
/// # Arguments
///
/// * `data` - A slice of bytes representing the binary data to parse.
/// * `offset` - The starting offset within the binary data for parsing Section1.
///
/// # Returns
///
/// * `anyhow::Result<()>` - Returns `Ok(())` if parsing is successful, or an
///   error wrapped in `anyhow::Error` if any part of the parsing fails.
///
/// # Errors
///
/// This function will return an error if:
/// - The binary data is invalid or incomplete.
/// - Parsing any of the sections fails.
///
/// # Debug Output
///
/// The function prints debug information about the parsed structures, including:
/// - The offset and details of Section1.
/// - The offset and details of Section2 (if present).
/// - Any recursive parsing of Section3 (if present).
///
/// # Example
///
/// ```rust
/// use fxr_binary_reader::fxr::parse_section_1_tree::parse_section1_tree;
/// use log::error;
/// // Example 1: Provide at least 16 bytes for Section1
/// let data: &[u8] = &[0x00; 16]; // 16 bytes of zero
/// let offset: u32 = 0x10;
/// if let Err(e) = parse_section1_tree(data, offset) {
///     error!("Failed to parse Section1 tree: {}", e);
/// }
///
/// // Example 2: Provide enough bytes for Section1, Section2, and Section3
/// let data: &[u8] = &[
///     0x00, 0x01, 0x02, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, 0x03, // Section1 (16 bytes)
///     0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B, 0x0C, 0x0D, 0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, // Section2 (16 bytes)
///     0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x1B, 0x1C, 0x1D, 0x1E, 0x1F, 0x20, 0x21, 0x22, 0x23, // Section3 (16 bytes)
/// ];
/// let offset: u32 = 0x00;
/// if let Err(e) = parse_section1_tree(data, offset) {
///     error!("Failed to parse Section1 tree: {}", e);
/// }
/// ```
pub fn parse_section1_tree(data: &[u8], offset: u32) -> anyhow::Result<ParsedSections> {
    let section1 = parse_struct::<Section1Container>(data, offset, "Section1")?;
    debug!("Section1 @ 0x{:08X}: {:#?}", offset, section1);

    let mut section2 = None;
    let mut section3 = None;

    if section1.section2_count > 0 {
        let section2_offset = section1.section2_offset;
        section2 = Some(parse_struct::<Section2Container>(
            data,
            section2_offset,
            "Section2",
        )?);
        debug!("Section2 @ 0x{:08X}: {:#?}", section2_offset, section2);

        if let Some(ref sec2) = section2 {
            if sec2.section3_count > 0 {
                section3 = Some(parse_section3_tree(
                    data,
                    sec2.section3_offset,
                    sec2.section3_count,
                )?);
            }
        }
    }

    Ok(ParsedSections {
        section1,
        section2,
        section3,
    })
}

#[derive(Debug)]
pub struct ParsedSections<'a> {
    pub section1: Ref<&'a [u8], Section1Container>,
    pub section2: Option<Ref<&'a [u8], Section2Container>>,
    pub section3: Option<Ref<&'a [u8], [Section3Entry]>>, // Assuming Section3 is a collection
}
