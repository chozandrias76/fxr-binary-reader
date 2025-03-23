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

pub fn parse_section7_nested<'a>(
    data: &'a [u8],
    container: &crate::fxr::Section7Container,
    label: &str,
) -> anyhow::Result<ParsedSection7Nested<'a>> {
    debug!("{}: Parsing Section7Container: {:#?}", label, container);

    let mut parsed_section7 = parse_section7_container(data, container, label).unwrap();

    parse_section7_section8_entries(data, container, label, &mut parsed_section7).unwrap();

    Ok(parsed_section7)
}

fn parse_section7_section8_entries<'a>(
    data: &'a [u8],
    container: &crate::fxr::Section7Container,
    label: &str,
    parsed_section7: &mut ParsedSection7Nested<'a>,
) -> Result<(), anyhow::Error> {
    if container.section8_count > 0 {
        debug!(
            "{}: Parsing Section8[] @ offset 0x{:08X}, count {}",
            label, container.section8_offset, container.section8_count
        );
        let section8_entry: Ref<&[u8], [Section8Entry]> = parse_section_slice::<Section8Entry>(
            data,
            container.section8_offset,
            container.section8_count,
            &format!("{label}::Section8[] @ 0x{:08X}:", container.section8_offset),
        )?;

        for (i, section8_entry) in section8_entry.iter().enumerate() {
            parse_section7_section8(data, label, i, section8_entry, parsed_section7)?;
        }
    };
    Ok(())
}

fn parse_section7_section8<'a>(
    data: &'a [u8],
    label: &str,
    i: usize,
    entry: &Section8Entry,
    parsed_section7: &mut ParsedSection7Nested<'a>,
) -> Result<(), anyhow::Error> {
    let mut parsed_section8 = ParsedSection8 {
        section11: Vec::new(),
        section9: Vec::new(),
    };
    parse_section8_section11_entries(data, label, i, entry, &mut parsed_section8)?;
    parse_section9_entries(data, label, i, entry, &mut parsed_section8)?;
    parsed_section7.section8.push(parsed_section8);
    Ok(())
}

fn parse_section9_entries<'a>(
    data: &'a [u8],
    label: &str,
    i: usize,
    section8_entry: &Section8Entry,
    parsed_section8: &mut ParsedSection8<'a>,
) -> Result<(), anyhow::Error> {
    if section8_entry.section9_count > 0 {
        debug!(
            "{}: Parsing Section8[{}]::Section9[] @ offset 0x{:08X}, count {}",
            label, i, section8_entry.section9_offset, section8_entry.section9_count
        );
        let section9_entries: Ref<&[u8], [Section9Entry]> = parse_section_slice::<Section9Entry>(
            data,
            section8_entry.section9_offset,
            section8_entry.section9_count,
            &format!(
                "{label}::Section8[{}]::Section9[] @ 0x{:08X}",
                i, section8_entry.section9_offset
            ),
        )?;

        for (j, s9_entry) in section9_entries.iter().enumerate() {
            parse_section8_section9_entry(data, label, i, j, s9_entry, parsed_section8)?;
        }
    };
    Ok(())
}

fn parse_section8_section9_entry<'a>(
    data: &'a [u8],
    label: &str,
    i: usize,
    j: usize,
    s9_entry: &Section9Entry,
    parsed_section8: &mut ParsedSection8<'a>,
) -> Result<(), anyhow::Error> {
    let mut parsed_section9 = ParsedSection9 {
        section11: Vec::new(),
    };
    parse_section9_section11_entries(data, label, i, j, s9_entry, &mut parsed_section9)?;
    parsed_section8.section9.push(parsed_section9);
    Ok(())
}

fn parse_section9_section11_entries<'a>(
    data: &'a [u8],
    label: &str,
    i: usize,
    j: usize,
    s9_entry: &Section9Entry,
    parsed_section9: &mut ParsedSection9<'a>,
) -> Result<(), anyhow::Error> {
    if s9_entry.section11_count > 0 {
        debug!(
            "{}: Parsing Section8[{}]::Section9[{}]::Section11[] @ offset 0x{:08X}, count {}",
            label, i, j, s9_entry.section11_offset, s9_entry.section11_count
        );
        let section11_entries: Ref<&[u8], [Section11Entry]> = parse_section_slice::<Section11Entry>(
            data,
            s9_entry.section11_offset,
            s9_entry.section11_count,
            &format!(
                "{label}::Section8[{}]::Section9[{}]::Section11[] @ 0x{:08X}",
                i, j, s9_entry.section11_offset
            ),
        )?;
        parsed_section9.section11.push(section11_entries);
    };
    Ok(())
}

fn parse_section8_section11_entries<'a>(
    data: &'a [u8],
    label: &str,
    i: usize,
    section8_entry: &Section8Entry,
    parsed_section8: &mut ParsedSection8<'a>,
) -> Result<(), anyhow::Error> {
    if section8_entry.section11_count > 0 {
        debug!(
            "{}: Parsing Section8[{}]::Section11[] @ offset 0x{:08X}, count {}",
            label, i, section8_entry.section11_offset, section8_entry.section11_count
        );
        let section11_entries: Ref<&[u8], [Section11Entry]> = parse_section_slice::<Section11Entry>(
            data,
            section8_entry.section11_offset,
            section8_entry.section11_count,
            &format!(
                "{label}::Section8[{}]::Section11[] @ 0x{:08X}",
                i, section8_entry.section11_offset
            ),
        )?;
        parsed_section8.section11.push(section11_entries);
    };
    Ok(())
}

fn parse_section7_container<'a>(
    data: &'a [u8],
    container: &crate::fxr::Section7Container,
    label: &str,
) -> Result<ParsedSection7Nested<'a>, anyhow::Error> {
    let mut parsed_section7 = ParsedSection7Nested {
        section11: Vec::new(),
        section8: Vec::new(),
    };
    parse_section7_section11_entries(data, container, label, &mut parsed_section7)?;
    Ok(parsed_section7)
}

fn parse_section7_section11_entries<'a>(
    data: &'a [u8],
    container: &crate::fxr::Section7Container,
    label: &str,
    parsed_section7: &mut ParsedSection7Nested<'a>,
) -> Result<(), anyhow::Error> {
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
    };
    Ok(())
}
