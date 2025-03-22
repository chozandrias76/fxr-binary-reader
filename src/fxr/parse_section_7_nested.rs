use log::debug;
use zerocopy::Ref;

use crate::fxr::util::parse_section_slice;
use crate::fxr::{Section8Entry, Section9Entry, Section11Entry};

pub struct ParsedSection9<'a> {
    pub section11: Vec<Ref<&'a [u8], [Section11Entry]>>,
}

pub struct ParsedSection8<'a> {
    pub section11: Vec<Ref<&'a [u8], [Section11Entry]>>,
    pub section9: Vec<ParsedSection9<'a>>,
}

pub struct ParsedSection7Nested<'a> {
    pub section11: Vec<Ref<&'a [u8], [Section11Entry]>>,
    pub section8: Vec<ParsedSection8<'a>>,
}

/// Parses Section7Container and its nested sections
/// # Arguments
/// * `data` - The byte slice containing the data to parse
/// * `container` - The Section7Container containing offsets and counts for nested sections
/// * `label` - A label for logging purposes
/// # Returns
/// * `Result` - Ok if parsing is successful, Err if there is an error
/// This function handles parsing nested sections within Section7, including Section11, Section8, and Section9.
/// It prints the parsed data to the console.
/// # Example
/// ```rust
/// fn main() -> anyhow::Result<()> {
///     use fxr_binary_reader::fxr::util::parse_struct;
///     use fxr_binary_reader::fxr::{Section7Container, parse_section_7_nested};
///     let data: &mut [u8] = &mut [0x0; 1000];
///     data[0x0..0x4].copy_from_slice(&[0x01, 0x00, 0x00, 0x00]);
///     data[0x4..0x8].copy_from_slice(&[0x01, 0x00, 0x00, 0x00]);
///     data[0x8..0xc].copy_from_slice(&[0x01, 0x00, 0x00, 0x00]);
///     data[0xc..0x10].copy_from_slice(&[0x01, 0x00, 0x00, 0x00]);
///     data[0x10..0x14].copy_from_slice(&[0x01, 0x00, 0x00, 0x00]);
///     data[0x14..0x18].copy_from_slice(&[0x01, 0x00, 0x00, 0x00]);
///     let data2: &mut [u8] = &mut [0x0; 1000];
///     let section7_offset = 0x20;
///     let container = parse_struct::<Section7Container>(data2, section7_offset, "")?;
///     Ok(())
/// }
///```
pub fn parse_section7_nested<'a>(
    data: &'a [u8],
    container: &crate::fxr::Section7Container,
    label: &str,
) -> anyhow::Result<ParsedSection7Nested<'a>> {
    debug!("{}: {:#?}", label, container);

    let mut parsed_section7 = ParsedSection7Nested {
        section11: Vec::new(),
        section8: Vec::new(),
    };

    // Section11[] from Section7Container
    if container.section11_count > 0 {
        let entries: Ref<&[u8], [Section11Entry]> = parse_section_slice::<Section11Entry>(
            data,
            container.section11_offset,
            container.section11_count,
            &format!(
                "{label}::Section11[] @ 0x{:08X}:",
                container.section11_offset
            ),
        )?;
        parsed_section7.section11.push(entries);
    }

    // Section8Entry[]
    if container.section8_count > 0 {
        let entries: Ref<&[u8], [Section8Entry]> = parse_section_slice::<Section8Entry>(
            data,
            container.section8_offset,
            container.section8_count,
            &format!("{label}::Section8[] @ 0x{:08X}:", container.section8_offset),
        )?;

        for (i, entry) in entries.iter().enumerate() {
            let mut parsed_section8 = ParsedSection8 {
                section11: Vec::new(),
                section9: Vec::new(),
            };

            // Section11[] inside Section8Entry
            if entry.section11_count > 0 {
                let s11: Ref<&[u8], [Section11Entry]> = parse_section_slice::<Section11Entry>(
                    data,
                    entry.section11_offset,
                    entry.section11_count,
                    &format!(
                        "{label}::Section8[{}]::Section11[] @ 0x{:08X}",
                        i, entry.section11_offset
                    ),
                )?;
                parsed_section8.section11.push(s11);
            }

            // Section9Entry[]
            if entry.section9_count > 0 {
                let s9: Ref<&[u8], [Section9Entry]> = parse_section_slice::<Section9Entry>(
                    data,
                    entry.section9_offset,
                    entry.section9_count,
                    &format!(
                        "{label}::Section8[{}]::Section9[] @ 0x{:08X}",
                        i, entry.section9_offset
                    ),
                )?;

                for (j, s9_entry) in s9.iter().enumerate() {
                    let mut parsed_section9 = ParsedSection9 {
                        section11: Vec::new(),
                    };

                    // Optional nested Section11[]
                    if s9_entry.section11_count > 0 {
                        let s11: Ref<&[u8], [Section11Entry]> =
                            parse_section_slice::<Section11Entry>(
                                data,
                                s9_entry.section11_offset,
                                s9_entry.section11_count,
                                &format!(
                                    "{label}::Section8[{}]::Section9[{}]::Section11[] @ 0x{:08X}",
                                    i, j, s9_entry.section11_offset
                                ),
                            )?;
                        parsed_section9.section11.push(s11);
                    }

                    parsed_section8.section9.push(parsed_section9);
                }
            }

            parsed_section7.section8.push(parsed_section8);
        }
    }

    Ok(parsed_section7)
}
