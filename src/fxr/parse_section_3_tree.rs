use zerocopy::Ref;

use crate::fxr::Section3Entry;
use crate::fxr::util::parse_section_slice;

/// Parses Section3 entries from the provided binary data.
///
/// This function reads a slice of binary data starting at the given offset and parses
/// `count` number of `Section3Entry` structures. It prints debug information about
/// each parsed entry, including its index, memory address, and contents.
///
/// # Arguments
///
/// * `data` - A slice of bytes representing the binary data to parse.
/// * `offset` - The starting offset within the binary data for parsing Section3 entries.
/// * `count` - The number of Section3 entries to parse.
///
/// # Returns
///
/// * `Ok(Ref<&[u8], [Section3Entry]>)` - A reference to the parsed Section3 entries if successful.
/// * `Err(anyhow::Error)` - An error if parsing fails.
///
/// # Errors
///
/// This function will return an error if:
/// - The provided offset and count exceed the bounds of the data slice.
/// - The binary data cannot be parsed into valid `Section3Entry` structures.
///
/// # Debug Output
///
/// For each parsed entry, the function prints:
/// - The index of the entry.
/// - The memory address relative to the start of the data slice.
/// - The contents of the entry.
///
/// # Example
///
/// ```rust
/// use fxr_binary_reader::fxr::parse_section_3_tree::parse_section3_tree;
/// use fxr_binary_reader::fxr::Section3Entry;
///
/// let data: &[u8] = &[0x0; 1000];
/// let offset: u32 = 0x10;
/// let count: u32 = 5;
///
/// let entries = parse_section3_tree(data, offset, count);
///
/// for entry in entries.iter() {
///   println!("{:?}", entry);
/// }
///
/// ```
pub fn parse_section3_tree(
    data: &[u8],
    offset: u32,
    count: u32,
) -> anyhow::Result<Ref<&[u8], [Section3Entry]>> {
    let entries = parse_section_slice::<Section3Entry>(data, offset, count, "Section3")?;
    for (i, entry) in entries.iter().enumerate() {
        let ptr = entry as *const _ as usize - data.as_ptr() as usize;
        println!("Section3[{}] @ 0x{:08X}: {:#?}", i, ptr, entry);
    }

    Ok(entries)
}
