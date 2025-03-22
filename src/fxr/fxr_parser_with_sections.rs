use memmap2::Mmap;
use std::{fs::File, path::Path};
use zerocopy::Ref;

use crate::fxr::Header;
use crate::fxr::parse_section_1_tree::parse_section1_tree;
use crate::fxr::parse_section_4_tree::parse_section4_tree;
use crate::fxr::util::parse_named_u32_entries;

pub fn parse_fxr(path: &Path) -> anyhow::Result<()> {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    let data = &mmap[..];
    let header_size = std::mem::size_of::<Header>();

    let header_ref = Ref::<_, Header>::from_bytes(&data[..header_size])
        .map_err(|_| anyhow::anyhow!("Failed to read header"))?;

    assert_eq!(&header_ref.magic, b"FXR\0");
    assert_eq!(header_ref.version, 5);

    println!("Header @ 0x00000000: {:#?}", header_ref);

    if header_ref.section1_count > 0 {
        parse_section1_tree(data, header_ref.section1_offset)?;
    }

    if header_ref.section4_count > 0 {
        parse_section4_tree(data, header_ref.section4_offset)?;
    }

    use crate::fxr::{Section12Entry, Section13Entry, Section14Entry};

    parse_named_u32_entries::<Section12Entry>(
        data,
        header_ref.section12_offset,
        header_ref.section12_count,
        "Section12",
    )?;

    parse_named_u32_entries::<Section13Entry>(
        data,
        header_ref.section13_offset,
        header_ref.section13_count,
        "Section13",
    )?;

    parse_named_u32_entries::<Section14Entry>(
        data,
        header_ref.section14_offset,
        header_ref.section14_count,
        "Section14",
    )?;

    Ok(())
}

mod tests {
    use std::path::PathBuf;

    use super::parse_fxr;

    #[test]
    fn test_parse_fxr() {
        let path = PathBuf::from("./f000302421.fxr");
        if let Err(e) = parse_fxr(&path) {
            eprintln!("Error parsing FXR: {}", e);
            panic!("Test failed");
        }
    }
}
