use crate::fxr::{
    Section7Container, Section10Container, Section11Entry,
    util::{ParseError, parse_section_slice, parse_struct},
};
use log::debug;
use validator::Validate;
use zerocopy::Ref;
pub mod parse_section_7_nested;
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
///
/// This function handles parsing nested sections within Section6, including Section11, Section10, and Section7.
/// It prints the parsed data to the console.
/// # Example
///```rust
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///   use fxr_binary_reader::fxr::Section6Entry;
///   use fxr_binary_reader::fxr::Section11Entry;
///   use fxr_binary_reader::fxr::parse_section_6_nested::parse_section6_nested;
///   use fxr_binary_reader::fxr::util::parse_section_slice;
///   let fixture_path = "../../fixtures/f000302420.fxr";
///   let data = std::fs::read(fixture_path).unwrap();
///   let section6_offset = 0x1E0;
///   let section6_count = 0x24;
///   let index = 0;
///   let p = 1;
///   let entry = &parse_section_slice::<Section6Entry>(
///       &data,
///       section6_offset,
///       section6_count,
///       &format!("Section6Entry[] @ 0x{:08X}", section6_offset),
///   )?[p];
///   assert_eq!(entry.section11_count1, 56, "Section6Entry[{}]::section11_count1 is 0", p);
///   let parsed = parse_section6_nested(&data, entry, index).unwrap();
///
///   let section11 = parsed.section11;
///   assert!(section11.is_some());
///   let section11 = section11.unwrap();
///   assert_eq!(section11.len(), 56, "Section11 length mismatch");
///  for (i, entry) in section11.iter().enumerate() {
///     let ptr = entry as *const _ as usize - data.as_ptr() as usize;
///     assert_eq!(ptr, 0x1458 + (0xE0*p) + (4*i), "{}", format!("Section11 entry offset mismatch at index {}", i));
///  }
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
        )?;
        container.validate()?;

        parsed_section6.section7 = Some(ParsedSection7 { container });
    } else {
        debug!(
            "  Skipping Section7 parsing for Section6[{}]: section7_count1 is 0",
            index
        );
    }

    Ok(parsed_section6)
}
