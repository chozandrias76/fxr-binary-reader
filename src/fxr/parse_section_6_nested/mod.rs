use crate::fxr::{
    Section7Container, Section10Container, Section11Entry,
    util::{ParseError, parse_section_slice, parse_struct},
};
use log::debug;
use zerocopy::Ref;
mod parse_section_7_nested;
use parse_section_7_nested::parse_section7_nested;

#[derive(Debug)]
pub struct ParsedSection6<'a> {
    pub section11: Option<Ref<&'a [u8], [Section11Entry]>>,
    pub section10: Option<ParsedSection10<'a>>,
    pub section7: Option<ParsedSection7<'a>>,
}

#[derive(Debug)]
pub struct ParsedSection10<'a> {
    pub container: Ref<&'a [u8], Section10Container>,
    pub section11: Option<Ref<&'a [u8], [Section11Entry]>>,
}

#[derive(Debug)]
pub struct ParsedSection7<'a> {
    pub container: Ref<&'a [u8], Section7Container>,
}

/// Parses nested sections within Section6
/// # Arguments
/// * `data` - The byte slice containing the data to parse
/// * `entry` - The Section6Entry containing offsets and counts for nested sections
/// * `index` - The index of the Section6 entry being parsed
/// # Returns
/// * `Result` - Ok if parsing is successful, Err if there is an error
/// This function handles parsing nested sections within Section6, including Section11, Section10, and Section7.
/// It prints the parsed data to the console.
/// # Example
///```rust
/// fn main() -> anyhow::Result<()> {
///   use fxr_binary_reader::fxr::Section6Entry;
///   use fxr_binary_reader::fxr::parse_section_6_nested::parse_section6_nested;
///   use fxr_binary_reader::fxr::util::parse_section_slice;
///   let data: &[u8] = &[
///     // Section6Entry (example values)
///     0x01, 0x00, // unk00 (u16)
///     0x02, // unk02 (u8)
///     0x03, // unk03 (u8)
///     0x10, 0x00, 0x00, 0x00, // unk04 (u32)
///     0x01, 0x00, 0x00, 0x00, // section11_count1 (u32)
///     0x01, 0x00, 0x00, 0x00, // section10_count (u32)
///     0x01, 0x00, 0x00, 0x00, // section7_count1 (u32)
///     0x00, 0x00, 0x00, 0x00, // section11_count2 (u32)
///     0x00, 0x00, 0x00, 0x00, // unk18 (u32)
///     0x01, 0x00, 0x00, 0x00, // section7_count2 (u32)
///     0x20, 0x00, 0x00, 0x00, // section11_offset (u32)
///     0x00, 0x00, 0x00, 0x00, // unk24 (u32)
///     0x30, 0x00, 0x00, 0x00, // section10_offset (u32)
///     0x00, 0x00, 0x00, 0x00, // unk2c (u32)
///     0x40, 0x00, 0x00, 0x00, // section7_offset (u32)
///     0x00, 0x00, 0x00, 0x00, // unk34 (u32)
///     0x00, 0x00, 0x00, 0x00, // unk38 (u32)
///     0x00, 0x00, 0x00, 0x00, // unk3c (u32)
///     // Section11Entry (example values)
///     0x00, 0x00, 0x00, 0x00, // unk00 (u32)
///     // Section10Container (example values)
///     0x01, 0x00, 0x00, 0x00, // unk04 (u32)
///     0x01, 0x00, 0x00, 0x00, // section11_offset (u32)
///     // Section7Container (example values)
///     0x00, 0x00, 0x00, 0x00, // unk0c (u32)
///     // Additional padding to ensure sufficient buffer size
///     100, 0x00, 0x00, 0x00, // section11_offset (u32)
///     0x00, 0x00, 0x00, 0x00, // Padding
///     108, 0x00, 0x00, 0x00, // section8_offset (u32)
///     0x00, 0x00, 0x00, 0x00, // Padding
///     1, 0x00, 0x00, 0x00, // section8_count (u32)
///     0x00, 0x00, 0x00, 0x00, // Padding
///     0x00, 0x00, 0x00, 0x00, // Padding
///     0x00, 0x00, 0x00, 0x00, // Padding
///     0x00, 0x00, 0x00, 0x00, // Padding
///     0x00, 0x00, 0x00, 0x00, // Padding
///     0x00, 0x00, 0x00, 0x00, // Padding
///     0x00, 0x00, 0x00, 0x00, // Padding
///     0x00, 0x00, 0x00, 0x00, // Padding
///     0x00, 0x00, 0x00, 0x00, // Padding
///     0x00, 0x00, 0x00, 0x00, // Padding
///     0x00, 0x00, 0x00, 0x00, // Padding
///     0x00, 0x00, 0x00, 0x00, // Padding
///   ];
///   let section6_offset = 0x00;
///   let section6_count = 1;
///   let index = 0;
///   let entry = &parse_section_slice::<Section6Entry>(
///       data,
///       section6_offset,
///       section6_count,
///       &format!("Section6Entry[] @ 0x{:08X}", section6_offset),
///   )?[0];
///   let parsed = parse_section6_nested(data, entry, index)?;
///
///   // Assert on key values
///   assert!(parsed.section11.is_some());
///   let section11 = parsed.section11.unwrap();
///   let section11 = section11.get(0).unwrap();
///   assert_eq!(section11.data, u32::from_ne_bytes([0x20, 0x00, 0x00, 0x00]));
///
///   assert!(parsed.section10.is_some());
///   let section10 = parsed.section10.unwrap();
///   let section10 = section10.container;
///   assert_eq!(section10.section11_offset, 0x40);
///   assert_eq!(section10.section11_count, 0x00);
///
///   assert!(parsed.section7.is_some());
///   let section7 = parsed.section7.unwrap();
///   let section7 = section7.container;
///   assert_eq!(section7.section11_count, 0x01, "section11_count should be 1");
///   assert_eq!(section7.section11_offset, 0x64, "section11_offset should be 100");
///
///   assert_eq!(section7.section8_count, 0x01, "section8_count should be 1");
///   assert_eq!(section7.section8_offset, 0x6C, "section8_offset should be 108");
///
///   Ok(())
/// }
/// ```
pub fn parse_section6_nested<'a>(
    data: &'a [u8],
    entry: &crate::fxr::Section6Entry,
    index: usize,
) -> Result<ParsedSection6<'a>, ParseError> {
    debug!("Parsing nested sections in Section6[{}]", index);

    let mut parsed_section6 = ParsedSection6 {
        section11: None,
        section10: None,
        section7: None,
    };

    // Validate Section11[] offsets and counts
    if entry.section11_count1 > 0 {
        let required_size = entry.section11_offset
            + entry.section11_count1 * (std::mem::size_of::<Section11Entry>() as u32);
        if (data.len() as u32) < required_size {
            return Err(ParseError::BufferTooSmall {
                expected: required_size as usize,
                actual: data.len(),
            });
        }

        let section11 = parse_section_slice::<Section11Entry>(
            data,
            entry.section11_offset,
            entry.section11_count1,
            &format!("Section6[{}]::Section11[]", index),
        )?;
        parsed_section6.section11 = Some(section11);
        for (i, v) in section11.iter().enumerate() {
            let ptr = v as *const _ as usize - data.as_ptr() as usize;
            debug!("  Section11[{}] @ 0x{:08X}: {:#?}", i, ptr, v);
        }
    } else {
        debug!(
            "  Skipping Section11[] parsing for Section6[{}]: section11_count1 is 0",
            index
        );
    }

    // Validate Section10 container
    if entry.section10_count > 0 {
        let required_size =
            entry.section10_offset + (std::mem::size_of::<Section10Container>() as u32);
        if (data.len() as u32) < required_size {
            return Err(ParseError::BufferTooSmall {
                expected: required_size as usize,
                actual: data.len(),
            });
        }

        let container = parse_struct::<Section10Container>(
            data,
            entry.section10_offset,
            &format!("Section6[{}]::Section10", index),
        )?;
        debug!(
            "  Section10 @ 0x{:08X}: {:#?}",
            entry.section10_offset, container
        );

        let mut parsed_section10 = ParsedSection10 {
            container,
            section11: None,
        };

        // Validate nested Section11[] in Section10
        if container.section11_count > 0 {
            let required_size = container.section11_offset
                + container.section11_count * (std::mem::size_of::<Section11Entry>() as u32);
            if (data.len() as u32) < required_size {
                return Err(ParseError::BufferTooSmall {
                    expected: required_size as usize,
                    actual: data.len(),
                });
            }

            let entries = parse_section_slice::<Section11Entry>(
                data,
                container.section11_offset,
                container.section11_count,
                &format!("Section6[{}]::Section10::Section11[]", index),
            )?;
            parsed_section10.section11 = Some(entries);
            for (i, entry) in entries.iter().enumerate() {
                let ptr = entry as *const _ as usize - data.as_ptr() as usize;
                debug!("  Section11[{}] @ 0x{:08X}: {:#?}", i, ptr, entry);
            }
        } else {
            debug!(
                "  Skipping nested Section11[] parsing in Section10 for Section6[{}]: section11_count is 0",
                index
            );
        }

        parsed_section6.section10 = Some(parsed_section10);
    } else {
        debug!(
            "  Skipping Section10 parsing for Section6[{}]: section10_count is 0",
            index
        );
    }

    // Validate Section7 container
    if entry.section7_count1 > 0 {
        let required_size =
            entry.section7_offset + (std::mem::size_of::<Section7Container>() as u32);
        if (data.len() as u32) < required_size {
            return Err(ParseError::BufferTooSmall {
                expected: required_size as usize,
                actual: data.len(),
            });
        }

        let container = parse_struct::<Section7Container>(
            data,
            entry.section7_offset,
            &format!(
                "Section6[{}]::Section7Container @ 0x{:08X}:",
                index, entry.section7_offset
            ),
        )?;
        let ptr = entry as *const _ as usize - data.as_ptr() as usize;
        parse_section7_nested(
            data,
            &container,
            &format!("Section6[{}]::Section7 @ 0x{:08X}", index, ptr),
        )
        .unwrap();

        parsed_section6.section7 = Some(ParsedSection7 { container });
    } else {
        debug!(
            "  Skipping Section7 parsing for Section6[{}]: section7_count1 is 0",
            index
        );
    }

    Ok(parsed_section6)
}
