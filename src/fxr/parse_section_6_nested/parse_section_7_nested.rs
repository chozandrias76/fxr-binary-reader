use crate::fxr::{Section8Entry, Section9Entry, Section11Entry, util::parse_section_slice};
use log::debug;
use zerocopy::Ref;

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
pub fn parse_section7_nested<'a>(
    data: &'a [u8],
    container: &crate::fxr::Section7Container,
    label: &str,
) -> anyhow::Result<ParsedSection7Nested<'a>> {
    debug!("{}: Parsing Section7Container: {:#?}", label, container);

    let mut parsed_section7 = ParsedSection7Nested {
        section11: Vec::new(),
        section8: Vec::new(),
    };

    // Section11[] from Section7Container
    if container.section11_count > 0 {
        debug!(
            "{}: Parsing Section11[] @ offset 0x{:08X}, count {}",
            label, container.section11_offset, container.section11_count
        );
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
        debug!(
            "{}: Parsing Section8[] @ offset 0x{:08X}, count {}",
            label, container.section8_offset, container.section8_count
        );
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
                debug!(
                    "{}: Parsing Section8[{}]::Section11[] @ offset 0x{:08X}, count {}",
                    label, i, entry.section11_offset, entry.section11_count
                );
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
                debug!(
                    "{}: Parsing Section8[{}]::Section9[] @ offset 0x{:08X}, count {}",
                    label, i, entry.section9_offset, entry.section9_count
                );
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
                        debug!(
                            "{}: Parsing Section8[{}]::Section9[{}]::Section11[] @ offset 0x{:08X}, count {}",
                            label, i, j, s9_entry.section11_offset, s9_entry.section11_count
                        );
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
