use crate::fxr::{
    Section4Container, Section4Entry, Section5Entry, Section6Entry,
    parse_section_6_nested::parse_section6_nested,
    util::{parse_section_slice, parse_struct},
};
use log::debug;
use zerocopy::Ref;

/// Parses a binary data structure starting at a given offset, extracting and printing details
/// about `Section4`, `Section5`, and `Section6` entries.
///
/// # Arguments
/// * `data` - A slice of bytes representing the binary data to be parsed.
/// * `offset` - The starting offset within the `data` slice where the parsing begins.
///
/// # Returns
/// * `Ok(())` if parsing is successful.
/// * An error if any part of the parsing process fails.
///
/// # Details
/// 1. Parses the `Section4Container` structure at the given offset and prints its details.
/// 2. If `section4_count` > 0, parses and prints an array of `Section4Entry` structures.
/// 3. If `section5_count` > 0, parses and prints an array of `Section5Entry` structures.
/// 4. If `section6_count` > 0, parses and prints an array of `Section6Entry` structures,
///    and further processes each entry using `parse_section6_nested`.
///
/// # Example Output
/// ```text
/// Section4Container @ 0x00000000: { ... }
/// Section4[0] @ 0x00000010: { ... }
/// Section5[0] @ 0x00000020: { ... }
/// Section6[0] @ 0x00000030: { ... }
/// ```
///
/// # Errors
/// This function may return an error if:
/// * The binary data is malformed or incomplete.
/// * Parsing any structure or slice fails.
///
///```rust
///
///
/// fn main() -> anyhow::Result<()> {
///     use std::mem;
///     use fxr_binary_reader::fxr::parse_section_4_tree::parse_section4_tree;
///     use fxr_binary_reader::fxr::Section4Container;
///     use fxr_binary_reader::fxr::Section4Entry;
///     use fxr_binary_reader::fxr::Section5Entry;
///     use fxr_binary_reader::fxr::Section6Entry;
///     // Sample binary data
///     let mut data = vec![0; mem::size_of::<Section4Container>()+mem::size_of::<Section4Entry>()+mem::size_of::<Section5Entry>()+mem::size_of::<Section6Entry>()];
///
///     // section5_count = 1
///     data[8..12].copy_from_slice(&1u32.to_le_bytes());
///
///     // section6_count = 1
///     data[12..16].copy_from_slice(&1u32.to_le_bytes());
///
///     // section4_count = 1
///     data[16..20].copy_from_slice(&1u32.to_le_bytes());
///
///     // section5_offset = 0x20
///     data[24..28].copy_from_slice(&0x20u32.to_le_bytes());
///
///     // section6_offset = 0x28
///     data[32..36].copy_from_slice(&0x28u32.to_le_bytes());
///
///     // section4_offset = 0x30
///     data[40..44].copy_from_slice(&0x30u32.to_le_bytes());
///
///
///     // Parse Section4 tree starting at offset 0
///     let section_tree = parse_section4_tree(&data, 0)?;
///     // Assert against tree structure
///     assert!(section_tree.section4_entries.is_some());
///     assert!(section_tree.section5_entries.is_some());
///     assert!(section_tree.section6_entries.is_some());
///
///     Ok(())
/// }
/// ```
pub fn parse_section4_tree(data: &[u8], offset: u32) -> anyhow::Result<ParsedSection4Tree> {
    let container = parse_struct::<Section4Container>(data, offset, "Section4Container")?;
    debug!("Section4Container @ 0x{:08X}: {:#?}", offset, container);

    let section4_entries = if container.section4_count > 0 {
        let entries = parse_section_slice::<Section4Entry>(
            data,
            container.section4_offset,
            container.section4_count,
            &format!("Section4Entry[] @ 0x{:08X}", container.section4_offset),
        )?;
        Some(entries)
    } else {
        None
    };

    let section5_entries = if container.section5_count > 0 {
        let entries = parse_section_slice::<Section5Entry>(
            data,
            container.section5_offset,
            container.section5_count,
            &format!("Section5Entry[] @ 0x{:08X}", container.section5_offset),
        )?;
        Some(entries)
    } else {
        None
    };

    let section6_entries = if container.section6_count > 0 {
        let entries = parse_section_slice::<Section6Entry>(
            data,
            container.section6_offset,
            container.section6_count,
            &format!("Section6Entry[] @ 0x{:08X}", container.section6_offset),
        )?;
        for (i, entry) in entries.iter().enumerate() {
            let ptr = entry as *const _ as usize - data.as_ptr() as usize;
            debug!("Section6[{}] @ 0x{:08X}: {:#?}", i, ptr, entry);
            parse_section6_nested(data, entry, i)?;
        }
        Some(entries)
    } else {
        None
    };

    Ok(ParsedSection4Tree {
        container,
        section4_entries,
        section5_entries,
        section6_entries,
    })
}

#[derive(Debug)]
pub struct ParsedSection4Tree<'a> {
    pub container: Ref<&'a [u8], Section4Container>,
    pub section4_entries: Option<zerocopy::Ref<&'a [u8], [Section4Entry]>>,
    pub section5_entries: Option<zerocopy::Ref<&'a [u8], [Section5Entry]>>,
    pub section6_entries: Option<zerocopy::Ref<&'a [u8], [Section6Entry]>>,
}
