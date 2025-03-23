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
/// Returns a ParsedSection7Nested struct containing parsed data
/// # Arguments
/// * `data` - The byte slice containing the data to parse
/// * `container` - The Section7Container to parse
/// * `label` - A label for logging purposes
/// # Returns
/// * `Result<ParsedSection7Nested<'a>, anyhow::Error>` - A result containing the parsed data or an error
/// # Example
/// ```
///  use fxr_binary_reader::{
///      fxr, fxr::parse_section_6_nested::parse_section_7_nested::parse_section7_nested,
///  };
///  let fixture_path = "fixtures/f000302420.fxr";
///  let data = std::fs::read(fixture_path).unwrap();
///  let mut container = fxr::Section7Container::default();
///  container.section11_count = 248;
///  container.section11_offset = 0x1530;
///  container.section8_offset = 0x1510;
///  let label = "TestLabel";
///  let parsed_data = parse_section7_nested(&data, &container, label).unwrap();
///  assert_eq!(
///      parsed_data.section11.len(),
///      1,
///      "Section11 should not be empty"
///  );
///  let section11_entry = &parsed_data.section11[0];
///  assert_eq!(
///      section11_entry.len(),
///      248,
///      "Section11 entry length should be 248"
///  );
///  for (i, entry) in section11_entry.iter().enumerate() {
///      if vec![0, 186, 203, 204, 205, 206, 224, 174, 175, 176, 230].contains(&i) {
///          assert_eq!(
///              entry.data,
///              0x3F800000,
///              "{}",
///              format!(
///                  "[{}] {}",
///                  i, "Section11 entry data should match expected value"
///              )
///          );
///      } else if {
///          let valid_indices: std::collections::HashSet<_> = (3..=5)
///              .chain(58..=122)
///              .chain(134..=135)
///              .chain(146..=146)
///              .chain(148..=148)
///              .chain(159..=160)
///              .chain(171..=171)
///              .chain(177..=179)
///              .chain(185..=185)
///              .chain(198..=198)
///              .chain(227..=227)
///              .chain(238..=238)
///              .collect();
///          valid_indices.contains(&i)
///      } {
///          assert_eq!(
///              entry.data,
///              0x00000001,
///              "{}",
///              format!(
///                  "[{}]{}",
///                  i, "Section11 entry data should match expected value"
///              )
///          );
///      } else if i == 25 {
///          assert_eq!(
///              entry.data,
///              0xFFFFFFFF,
///              "{}",
///              format!(
///                  "[{}]{}",
///                  i, "Section11 entry data should match expected value"
///              )
///          );
///      } else if vec![137, 147, 162, 172].contains(&i) {
///          assert_eq!(
///              entry.data,
///              0xBD088889,
///              "{}",
///              format!(
///                  "[{}]{}",
///                  i, "Section11 entry data should match expected value"
///              )
///          );
///      } else if i == 173 {
///          assert_eq!(
///              entry.data,
///              0x00000002,
///              "{}",
///              format!(
///                  "[{}]{}",
///                  i, "Section11 entry data should match expected value"
///              )
///          );
///      } else if vec![181, 182, 234, 235].contains(&i) {
///          assert_eq!(
///              entry.data,
///              0xFFFFFFFE,
///              "{}",
///              format!(
///                  "[{}]{}",
///                  i, "Section11 entry data should match expected value"
///              )
///          );
///      } else if vec![201].contains(&i) {
///          assert_eq!(
///              entry.data,
///              0x00000008,
///              "{}",
///              format!(
///                  "[{}]{}",
///                  i, "Section11 entry data should match expected value"
///              )
///          );
///      } else if vec![237].contains(&i) {
///          assert_eq!(
///              entry.data,
///              0x47AA0A00,
///              "{}",
///              format!(
///                  "[{}]{}",
///                  i, "Section11 entry data should match expected value"
///              )
///          );
///      } else if vec![233].contains(&i) {
///          assert_eq!(
///              entry.data,
///              0x3F000000,
///              "{}",
///              format!(
///                  "[{}]{}",
///                  i, "Section11 entry data should match expected value"
///              )
///          );
///      } else if vec![213, 214, 215, 216, 217, 218].contains(&i) {
///          assert_eq!(
///              entry.data,
///              0xBF800000,
///              "{}",
///              format!(
///                  "[{}]{}",
///                  i, "Section11 entry data should match expected value"
///              )
///          );
///      } else {
///          assert_eq!(
///              entry.data,
///              0x00000000,
///              "{}",
///              format!(
///                  "[{}]{}",
///                  i, "Section11 entry data should match expected value"
///              )
///          );
///      }
///  }
///  assert_eq!(parsed_data.section8.len(), 0, "Section8 should be empty");
///```
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
