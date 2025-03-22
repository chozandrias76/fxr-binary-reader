use super::U32Field;
use log::debug;
use zerocopy::{FromBytes, Immutable, KnownLayout, Ref};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Data buffer is too small: expected at least {expected} bytes, got {actual} bytes")]
    BufferTooSmall { expected: usize, actual: usize },
    #[error("Offset {offset} plus size {size} exceeds data length {data_len}")]
    OutOfBounds {
        offset: usize,
        size: usize,
        data_len: usize,
    },
    #[error("Size overflow: entry_size={entry_size}, count={count}")]
    SizeOverflow { entry_size: usize, count: usize },
    #[error(
        "Failed to parse {label}: start={start}, end={end}, entry_size={entry_size}, count={count}"
    )]
    ParseFailed {
        label: String,
        start: usize,
        end: usize,
        entry_size: usize,
        count: usize,
    },
}

/// Parses a list of named `u32` entries from a data buffer.
///
/// This function extracts and processes a list of entries of type `T` from the provided data buffer.
/// It validates the input parameters, ensures the entries are within bounds, and prints detailed
/// information about each entry.
///
/// # Type Parameters
/// - `T`: The type of the entries to parse. Must implement `FromBytes`, `KnownLayout`, `Immutable`, and `U32Field`.
///
/// # Arguments
/// - `data`: A reference to the data buffer containing the entries.
/// - `offset`: The starting offset (in bytes) of the entries within the data buffer.
/// - `count`: The number of entries to parse.
/// - `label`: A label used for logging and error messages to identify the context of the operation.
///
/// # Returns
/// - `Ok(())`: If the entries are successfully parsed and processed.
/// - `Err(anyhow::Error)`: If an error occurs during parsing.
///
/// # Errors
/// - Returns an error if the `count` is zero but the function is called unnecessarily.
/// - Returns an error if the data buffer is too small or the offset and count exceed its bounds.
/// - Returns an error if the entries cannot be parsed into the specified type `T`.
///
/// # Examples
/// ```rust
/// use zerocopy_derive::{FromBytes, KnownLayout, Immutable};
/// use anyhow::Result;
/// use fxr_binary_reader::fxr::util::parse_named_u32_entries;
/// use fxr_binary_reader::fxr::U32Field;
///
/// #[repr(C)]
/// #[derive(FromBytes, KnownLayout, Immutable, Debug)]
/// struct MyEntry {
///     value: u32,
/// }
///
/// impl U32Field for MyEntry {
///     fn data(&self) -> u32 {
///         self.value
///     }
/// }
///
/// fn main() -> Result<()> {
///     let data: &[u8] = &[
///         0x01, 0x00, 0x00, 0x00, // Entry 1
///         0x02, 0x00, 0x00, 0x00, // Entry 2
///     ];
///     let _entries = parse_named_u32_entries::<MyEntry>(data, 0, 2, "TestEntries")?;
///     Ok(())
/// }
/// ```
pub fn parse_named_u32_entries<'a, T>(
    data: &'a [u8],
    offset: u32,
    count: u32,
    label: &str,
) -> anyhow::Result<Ref<&'a [u8], [T]>>
where
    T: FromBytes + KnownLayout + Immutable + U32Field,
{
    // if count == 0 { ///TODO: Figure out if this should be used on other fxr
    //     return Err(anyhow::anyhow!("{label} is empty. No entries to parse."));
    // }

    let entries = parse_section_slice::<T>(data, offset, count, label)
        .map_err(|e| anyhow::anyhow!("Failed to parse {label} entries: {e}"))?;

    debug!("{label} entries ({}):", entries.len());
    for (i, entry) in entries.iter().enumerate() {
        let ptr = entry as *const _ as usize;
        debug!(
            "  {}[{}] @ 0x{:08X} = 0x{:08X}",
            label,
            i,
            ptr,
            entry.data()
        );
    }

    Ok(entries)
}

/// Parses a struct from the given data buffer at the specified offset.
///
/// This function extracts a struct of type `T` from the provided data buffer, starting at the given
/// `offset`. It ensures that the struct is within bounds and can be parsed correctly.
///
/// # Type Parameters
/// - `T`: The type of the struct to parse. Must implement `FromBytes`, `KnownLayout`, and `Immutable`.
/// # Arguments
/// - `data`: A reference to the data buffer from which the struct will be extracted.
/// - `offset`: The starting offset (in bytes) of the struct within the data buffer.
/// - `label`: A label used for error messages to identify the context of the operation.
/// # Returns
/// - `Ok(Ref<&'a [u8], T>)`: A reference to the parsed struct of type `T`.
/// - `Err(anyhow::Error)`: An error if the struct is out of bounds or cannot be parsed.
/// # Errors
/// - Returns an error if the calculated end of the struct exceeds the length of the data buffer.
/// - Returns an error if the struct cannot be parsed into the specified type `T`.
/// Example usage of `parse_struct`:
/// ```rust
/// use zerocopy::{FromBytes, KnownLayout, Ref};
/// use anyhow::Result;
/// use fxr_binary_reader::fxr::util::parse_struct;
/// use zerocopy_derive::{Immutable, KnownLayout, FromBytes};
/// use log::debug;
///
/// #[repr(C)]
/// #[derive(FromBytes, KnownLayout, Immutable, Debug)]
/// struct MyStruct {
///     group1: Group1, // 4 bytes
///     group2: Group2, // 16 bytes
///     group3: Group3, // 8 bytes
///     group4: Group4, // 24 bytes
/// }
///
/// #[repr(C)]
/// #[derive(FromBytes, KnownLayout, Immutable, Debug)]
/// struct Group1 {
///     field1: u8,
///     _pad1: [u8; 1], // Padding to align field2
///     field2: i16,
/// }
///
/// #[repr(C)]
/// #[derive(FromBytes, KnownLayout, Immutable, Debug)]
/// struct Group2 {
///     field3: i8,
///     _pad2: [u8; 3], // Padding to align field4
///     field4: u32,
///     field5: u32,
///     field6: u32,
/// }
///
/// #[repr(C)]
/// #[derive(FromBytes, KnownLayout, Immutable, Debug)]
/// struct Group3 {
///     field7: i8,
///     _pad3: [u8; 3], // Padding to align field8
///     field8: u8,
///     field9: u8,
///     field10: u8,
///     _pad4: [u8; 1], // Padding to align field11
/// }
///
/// #[repr(C)]
/// #[derive(FromBytes, KnownLayout, Immutable, Debug)]
/// struct Group4 {
///     field11: u32,
///     field12: i64,
///     field13: u32,
///     field14: u32,
/// }
///
///fn main() -> anyhow::Result<()> {
///  let data: &[u8] = &[
///      0x01, 0x00,             // field1: u8 + padding
///      0x02, 0x00,             // field2: i16 (2 in little-endian)
///      0x03, 0x00, 0x00, 0x00, // field3: i8 + padding
///      0x04, 0x00, 0x00, 0x00, // field4: u32 (4 in little-endian)
///      0x05, 0x00, 0x00, 0x00, // field5: u32 (5 in little-endian)
///      0x06, 0x00, 0x00, 0x00, // field6: u32 (6 in little-endian)
///      0x07, 0x00, 0x00, 0x00, // field7: i8 + padding
///      0x08, 0x09, 0x0A, 0x00, // field8, field9, field10
///      0x00, 0x00, 0x00, 0x00, // padding
///      0x0B, 0x00, 0x00, 0x00, // field11: u32 (11 in little-endian)
///      0x00, 0x00, 0x00, 0x00,
///      0x0C, 0x00, 0x00, 0x00,
///      0x00, 0x00, 0x00, 0x00,
///      0x0D, 0x00, 0x00, 0x00,
///      0x0E, 0x00, 0x00, 0x00,
///  ];
///
///  let entry = parse_struct::<MyStruct>(data, 0, "MyStruct")?;
///  assert_eq!(entry.group1.field1, 1, "{}",format!("field1 was {}", entry.group1.field1).as_str());
///  assert_eq!(entry.group1.field2, 2, "{}",format!("field2 was {}", entry.group1.field2).as_str());
///  assert_eq!(entry.group2.field3, 3, "{}",format!("field3 was {}", entry.group2.field3).as_str());
///  assert_eq!(entry.group2.field4, 4, "{}",format!("field4 was {}", entry.group2.field4).as_str());
///  assert_eq!(entry.group2.field5, 5, "{}",format!("field5 was {}", entry.group2.field5).as_str());
///  assert_eq!(entry.group2.field6, 6, "{}",format!("field6 was {}", entry.group2.field6).as_str());
///  assert_eq!(entry.group3.field7, 7, "{}",format!("field7 was {}", entry.group3.field7).as_str());
///  assert_eq!(entry.group3.field8, 8, "{}",format!("field8 was {}", entry.group3.field8).as_str());
///  assert_eq!(entry.group3.field9, 9, "{}",format!("field9 was {}", entry.group3.field9).as_str());
///  assert_eq!(entry.group3.field10, 10, "{}", format!("field10 was {}", entry.group3.field10).as_str());
///  assert_eq!(entry.group4.field11, 11, "{}", format!("field11 was {}", entry.group4.field11).as_str());
///  assert_eq!(entry.group4.field12, 12, "{}", format!("field12 was {}", entry.group4.field12).as_str());
///  assert_eq!(entry.group4.field13, 13, "{}", format!("field13 was {}", entry.group4.field13).as_str());
///  assert_eq!(entry.group4.field14, 14, "{}", format!("field14 was {}", entry.group4.field14).as_str());
///
///  debug!("Parsed struct: {:?}", entry);
///  Ok(())
///}
///```
pub fn parse_struct<'a, T: FromBytes + KnownLayout + Immutable>(
    data: &'a [u8],
    offset: u32,
    label: &str,
) -> Result<Ref<&'a [u8], T>, ParseError> {
    let size = std::mem::size_of::<T>();
    debug!("Struct size: {}", size);
    debug!("Data length: {}", data.len());
    debug!("Offset: {}", offset);

    if data.len() < size {
        return Err(ParseError::BufferTooSmall {
            expected: size,
            actual: data.len(),
        });
    } else if offset as usize + size > data.len() {
        return Err(ParseError::OutOfBounds {
            offset: offset as usize,
            size,
            data_len: data.len(),
        });
    }

    let end = offset as usize + size;
    debug!("End index: {}", end);

    // Ensure the slice length matches the expected struct size
    if end > data.len() {
        return Err(ParseError::BufferTooSmall {
            expected: end,
            actual: data.len(),
        });
    }

    // Attempt to parse the struct
    let slice = &data[offset as usize..end];
    debug!("Slice length: {}, Slice: {:02X?}", slice.len(), slice);

    Ref::from_bytes(slice).map_err(|_| ParseError::ParseFailed {
        label: label.to_string(),
        start: offset as usize,
        end,
        entry_size: size,
        count: 1,
    })
}

/// Parses a slice of a section from the given data buffer.
///
/// This function extracts a slice of type `T` from the provided data buffer, starting at the given
/// `offset` and containing `count` elements. It ensures that the slice is within bounds and that
/// the size calculations do not overflow.
///
/// # Type Parameters
/// - `T`: The type of the elements in the slice. Must implement `FromBytes`, `KnownLayout`, and `Immutable`.
///
/// # Arguments
/// - `data`: A reference to the data buffer from which the slice will be extracted.
/// - `offset`: The starting offset (in bytes) of the slice within the data buffer.
/// - `count`: The number of elements of type `T` to extract.
/// - `label`: A label used for error messages to identify the context of the operation.
///
/// # Returns
/// - `Ok(Ref<&'a [u8], [T]>)`: A reference to the parsed slice of type `T`.
/// - `Err(anyhow::Error)`: An error if the slice is out of bounds, if size calculations overflow,
///   or if the slice cannot be parsed.
///
/// # Errors
/// - Returns an error if the calculated end of the slice exceeds the length of the data buffer.
/// - Returns an error if the size calculation overflows.
/// - Returns an error if the slice cannot be parsed into the specified type `T`.
///
/// # Examples
/// ```rust
/// use zerocopy::{FromBytes, KnownLayout, Ref};
/// use anyhow::Result;
/// use fxr_binary_reader::fxr::util::parse_section_slice;
/// use zerocopy_derive::{Immutable, KnownLayout, FromBytes};
/// use log::debug;
///
/// #[repr(C)]
/// #[derive(FromBytes, KnownLayout, Immutable, Debug)]
/// struct MyStruct {
///     field1: u8,
///     _pad1: [u8; 1],
///     field2: i16,
///     field3: i8,
///     _pad2: [u8; 3],
///     field4: u32,
///     field5: u32,
///     field6: u32,
///     field7: i8,
///     _pad3: [u8; 3],
///     field8: u8,
///     field9: u8,
///     field10: u8,
///     _pad4: [u8; 1],
///     field11: u32,
///     field12: i64,
///     field13: u32,
///     field14: u32,
/// }
///
/// fn main() -> Result<()> {
///     let data: &[u8] = &[
///         0x01, 0x00,             // field1: u8 + padding
///         0x02, 0x00,             // field2: i16 (2 in little-endian)
///         0x03, 0x00, 0x00, 0x00, // field3: i8 + padding
///         0x04, 0x00, 0x00, 0x00, // field4: u32 (4 in little-endian)
///         0x05, 0x00, 0x00, 0x00, // field5: u32 (5 in little-endian)
///         0x06, 0x00, 0x00, 0x00, // field6: u32 (6 in little-endian)
///         0x07, 0x00, 0x00, 0x00, // field7: i8 + padding
///         0x08, 0x09, 0x0A, 0x00, // field8, field9, field10 + padding
///         0x0B, 0x00, 0x00, 0x00, // field11: u32 (11 in little-endian)
///         0x0C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // field12: i64 (12 in little-endian)
///         0x0D, 0x00, 0x00, 0x00, // field13: u32 (13 in little-endian)
///         0x0E, 0x00, 0x00, 0x00, // field14: u32 (14 in little-endian)
///     ];
///     let slice = parse_section_slice::<MyStruct>(data, 0, 1, "MyStruct")?;
///     assert_eq!(slice.len(), 1);
///
///     let entry = &slice[0];
///     assert_eq!(entry.field1, 1);
///     assert_eq!(entry.field2, 2);
///     assert_eq!(entry.field3, 3);
///     assert_eq!(entry.field4, 4);
///     assert_eq!(entry.field5, 5);
///     assert_eq!(entry.field6, 6);
///     assert_eq!(entry.field7, 7);
///     assert_eq!(entry.field8, 8);
///     assert_eq!(entry.field9, 9);
///     assert_eq!(entry.field10, 10);
///     assert_eq!(entry.field11, 11);
///     assert_eq!(entry.field12, 12);
///     assert_eq!(entry.field13, 13);
///     assert_eq!(entry.field14, 14);
///
///     debug!("Parsed slice: {:?}", slice);
///     Ok(())
/// }
/// ```
pub fn parse_section_slice<'a, T: FromBytes + KnownLayout + Immutable>(
    data: &'a [u8],
    offset: u32,
    count: u32,
    label: &str,
) -> Result<Ref<&'a [u8], [T]>, ParseError> {
    let entry_size = std::mem::size_of::<T>();
    let start = offset as usize;
    let total_size = entry_size
        .checked_mul(count as usize)
        .ok_or(ParseError::SizeOverflow {
            entry_size,
            count: count as usize,
        })?;
    let end = start
        .checked_add(total_size)
        .ok_or(ParseError::SizeOverflow {
            entry_size,
            count: count as usize,
        })?;

    if end > data.len() {
        return Err(ParseError::OutOfBounds {
            offset: start,
            size: total_size,
            data_len: data.len(),
        });
    }

    Ref::<_, [T]>::from_bytes_with_elems(&data[start..end], count as usize).map_err(|_| {
        ParseError::ParseFailed {
            label: label.to_string(),
            start,
            end,
            entry_size,
            count: count as usize,
        }
    })
}
