use gimli::{AttributeValue, DebuggingInformationEntry, ReaderOffset, UnitOffset};
use gimli::{EndianSlice, EntriesCursor, Reader, Unit};
use object::{File, Object, ObjectSection, ReadRef};
use std::borrow::Cow;
use std::collections::BTreeMap;

use crate::r#type::{StructureMember, Type};
use crate::Error;
use crate::Result;

macro_rules! get_attribute {
    ($entry:ident, $attribute:path) => {
        $entry
            .attr_value($attribute)?
            .ok_or(Error::NoAttribute($attribute))?
    };
}

/// Contains an uncompressed dwarf and it's endianness.
#[derive(Debug)]
pub(crate) struct Dwarf<'data> {
    dwarf: gimli::Dwarf<Cow<'data, [u8]>>,
    endian: gimli::RunTimeEndian,
}

impl<'data> Dwarf<'data> {
    /// Creates a new [`Dwarf`]
    pub(crate) fn new<R: ReadRef<'data>>(file: &File<'data, R>) -> Result<Self> {
        let endian = if file.is_little_endian() {
            gimli::RunTimeEndian::Little
        } else {
            gimli::RunTimeEndian::Big
        };

        // We want the uncompressed sections, this returns a [`Cow`] that we have to store somewhere
        // for future use.
        // Unfortunately it's impossible to directly convert this Cow into an EndianSlice and store
        // only the endian slice, so we'll have to perform the conversion every time we need.
        let load_section = |id: gimli::SectionId| -> Result<std::borrow::Cow<[u8]>> {
            match file.section_by_name(id.name()) {
                Some(ref section) => Ok(section.uncompressed_data()?),
                None => Ok(std::borrow::Cow::Borrowed(&[][..])),
            }
        };

        let dwarf = gimli::Dwarf::load(&load_section)?;

        // Load all of the sections.
        Ok(Self { dwarf, endian })
    }

    /// Returns the loaded dwarf's endianness.
    pub(crate) fn endian(&self) -> gimli::RunTimeEndian {
        self.endian
    }

    /// Tries to find the type whose name is `type_name` and is located in the compilation unit
    /// `compilation_unit_name`.
    ///
    /// Output:
    /// * Returns `Ok(Some)` if the type is successfully found.
    /// * Returns `Ok(None)` if the type cannot be found.
    /// * Returns `Err` if an error is encountered.
    pub(crate) fn get_type(
        &'data self,
        compilation_unit_name: &str,
        type_name: &str,
    ) -> Result<Option<Type>> {
        let dwarf = self.borrow();
        let compilation_unit =
            if let Some(c) = find_compilation_unit(&dwarf, compilation_unit_name)? {
                c
            } else {
                return Ok(None);
            };

        let unit_offset = if let Some(c) = find_type_die(&dwarf, &compilation_unit, type_name)? {
            c
        } else {
            return Ok(None);
        };

        parse_type(&dwarf, &compilation_unit, unit_offset).map(Some)
    }

    /// Converts self into an EndianSlice Dwarf.
    fn borrow(&'data self) -> gimli::Dwarf<EndianSlice<'data, gimli::RunTimeEndian>> {
        // Borrow a `Cow<[u8]>` to create an `EndianSlice`.
        let borrow_section =
            |section: &'data Cow<'_, [u8]>| gimli::EndianSlice::new(section, self.endian);

        self.dwarf.borrow(&borrow_section)
    }
}

/// Tries to find the compilation unit by name.
///
/// Output:
/// * Returns `Ok(Some)` if the compilation unit is successfully found.
/// * Returns `Ok(None)` if the compilation unit cannot be found.
/// * Returns `Err` if an error is encountered.
fn find_compilation_unit<R: Reader>(
    dwarf: &gimli::Dwarf<R>,
    name: &str,
) -> Result<Option<Unit<R>>> {
    // Iterate over all the unit headers.
    let mut iter = dwarf.units();
    while let Some(header) = iter.next()? {
        let unit = dwarf.unit(header)?;

        // Iterate over the Debugging Information Entries (DIEs) in the unit header, usually the
        // the first one is the compilation unit DIE.
        let mut entries = unit.entries();
        while let Some((_, entry)) = entries.next_dfs()? {
            if entry.tag() != gimli::DW_TAG_compile_unit {
                continue;
            }

            let name_attribute = get_attribute!(entry, gimli::DW_AT_name);
            let unit_name = dwarf.attr_string(&unit, name_attribute)?;
            let unit_name = unit_name.to_string()?;

            if name == unit_name {
                return Ok(Some(unit));
            }
        }
    }

    Ok(None)
}

/// Tries to find the DIE representing the type in the provided compilation unit.
/// Returns the DIE's offset in the unit.
///
/// Output:
/// * Returns `Ok(Some)` if the type DIE is successfully found.
/// * Returns `Ok(None)` if the type DIE cannot be found.
/// * Returns `Err` if an error is encountered.
fn find_type_die<R: Reader>(
    dwarf: &gimli::Dwarf<R>,
    compilation_unit: &Unit<R>,
    type_name: &str,
) -> Result<Option<UnitOffset<R::Offset>>> {
    // Iterate over the Debugging Information Entries (DIEs) in the unit.
    let mut entries = compilation_unit.entries();
    while let Some((_, entry)) = entries.next_dfs()? {
        // Check each DIE's name.
        if let Some(name_attribute) = entry.attr_value(gimli::DW_AT_name)? {
            let name = dwarf.attr_string(compilation_unit, name_attribute)?;
            let name = name.to_string()?;

            if name == type_name {
                // Found our DIE.
                return Ok(Some(entry.offset()));
            }
        }
    }

    Ok(None)
}

/// Parses the type whose description starts at the provided offset.
///
/// Output:
/// * Returns `Ok` if the type DIE is successfully parsed.
/// * Returns `Err` if an error is encountered.
fn parse_type<R: Reader>(
    dwarf: &gimli::Dwarf<R>,
    unit: &Unit<R>,
    start_offset: UnitOffset<R::Offset>,
) -> Result<Type> {
    let mut entries = unit.entries_at_offset(start_offset)?;

    if let Some((_, entry)) = entries.next_dfs()? {
        let tag = entry.tag();

        // Parse known types
        match tag {
            gimli::DW_TAG_base_type => parse_base(entry),
            gimli::DW_TAG_enumeration_type => parse_enumeration(dwarf, unit, entries),
            gimli::DW_TAG_pointer_type => parse_pointer(entry),
            gimli::DW_TAG_structure_type => parse_structure(dwarf, unit, entries),
            gimli::DW_TAG_array_type => parse_array(dwarf, unit, entries),
            gimli::DW_TAG_const_type | gimli::DW_TAG_typedef => {
                let type_ref = get_attribute!(entry, gimli::DW_AT_type);

                if let AttributeValue::UnitRef(unit_ref) = type_ref {
                    parse_type(dwarf, unit, unit_ref)
                } else {
                    Err(Error::NoNullTerm)
                }
            }
            _ => Err(Error::UnexpectedTag(tag)),
        }
    } else {
        Err(Error::NoDIE(start_offset.0.into_u64()))
    }
}

/// Parses the enumeration type whose DIE is pointed to by the entries cursor.
///
/// Output:
/// * Returns `Ok` if the enumeration type DIE is successfully parsed.
/// * Returns `Err` if an error is encountered.
fn parse_enumeration<R: Reader>(
    dwarf: &gimli::Dwarf<R>,
    unit: &Unit<R>,
    mut entries: EntriesCursor<'_, '_, R>,
) -> Result<Type> {
    // Figure out the type of the storage used by the enum.
    // Unwrap safety: this function is called by `parse_type`, so the current entry must exist.
    let entry = entries.current().unwrap();
    let ty = parse_enumeration_storage(dwarf, unit, entry)?;
    let mut valid_values = BTreeMap::default();

    // Step into member DIEs
    if let Some((1, mut entry)) = entries.next_dfs()? {
        // Iterate over all the siblings (DW_TAG_enumerator DIEs)
        loop {
            let name = get_attribute!(entry, gimli::DW_AT_name);
            let name = dwarf.attr_string(unit, name)?;
            let name = name.to_string()?;

            let value = match ty {
                Type::I8 | Type::I16 | Type::I32 | Type::I64 => entry
                    .attr_value(gimli::DW_AT_const_value)?
                    .and_then(|o| o.sdata_value())
                    .unwrap()
                    as i128,
                Type::U8 | Type::U16 | Type::U32 | Type::U64 => entry
                    .attr_value(gimli::DW_AT_const_value)?
                    .and_then(|o| o.udata_value())
                    .unwrap()
                    as i128,
                _ => unreachable!("C enums must have integer types!"),
            };

            valid_values.insert(value, name.to_string());

            entry = if let Some(e) = entries.next_sibling()? {
                e
            } else {
                // Reached end of DW_TAG_enumerator DIEs
                break;
            };
        }
    }

    Ok(Type::Enumeration {
        ty: Box::new(ty),
        valid_values,
    })
}

/// Tries to determine an enumerator's storage type.
///
/// Output:
/// * Returns `Ok` if the enumeration type DIE is successfully parsed.
/// * Returns `Err` if an error is encountered.
fn parse_enumeration_storage<R: Reader>(
    dwarf: &gimli::Dwarf<R>,
    unit: &Unit<R>,
    entry: &DebuggingInformationEntry<'_, '_, R>,
) -> Result<Type> {
    if let Some(AttributeValue::UnitRef(unit_offset)) = entry.attr_value(gimli::DW_AT_type)? {
        parse_type(dwarf, unit, unit_offset)
    } else {
        // If the entry doesn't have a type attribute, try parsing it's encoding and size
        // attributes, like a base type.
        parse_base(entry)
    }
}

/// Parses the structure type whose DIE is pointed to by the entries cursor.
///
/// Output:
/// * Returns `Ok` if the structure type DIE is successfully parsed.
/// * Returns `Err` if an error is encountered.
fn parse_structure<R: Reader>(
    dwarf: &gimli::Dwarf<R>,
    unit: &Unit<R>,
    mut entries: EntriesCursor<'_, '_, R>,
) -> Result<Type> {
    let mut members = vec![];

    // Step into member DIEs
    if let Some((1, mut entry)) = entries.next_dfs()? {
        // Iterate over all the siblings until there's no more.
        loop {
            // Get the name of the member.
            let name = get_attribute!(entry, gimli::DW_AT_name);
            let name = dwarf.attr_string(unit, name)?;
            let name = name.to_string()?.to_string();

            // Get the type of the member.
            let ty = if let AttributeValue::UnitRef(unit_offset) =
                get_attribute!(entry, gimli::DW_AT_type)
            {
                parse_type(dwarf, unit, unit_offset)?
            } else {
                return Err(Error::BadAttribute);
            };

            // Get the members offset from the struct's beginning.
            let offset = entry
                .attr_value(gimli::DW_AT_data_member_location)?
                .and_then(|o| o.udata_value())
                .unwrap_or(0);

            members.push(StructureMember { name, ty, offset });

            // Get next sibling or break iteration.
            entry = if let Some(e) = entries.next_sibling()? {
                e
            } else {
                break;
            };
        }
    }

    Ok(Type::Structure(members))
}

/// Parses the array type whose DIE is pointed to by the entries cursor.
///
/// Output:
/// * Returns `Ok` if the array type DIE is successfully parsed.
/// * Returns `Err` if an error is encountered.
fn parse_array<R: Reader>(
    dwarf: &gimli::Dwarf<R>,
    unit: &Unit<R>,
    mut entries: EntriesCursor<'_, '_, R>,
) -> Result<Type> {
    let entry = entries.current().unwrap();
    let ty =
        if let Some(AttributeValue::UnitRef(unit_offset)) = entry.attr_value(gimli::DW_AT_type)? {
            parse_type(dwarf, unit, unit_offset)
        } else {
            // If the entry doesn't have a type attribute, try parsing it's encoding and size
            // attributes, like a base type.
            parse_base(entry)
        }?;

    let mut lengths = vec![];

    // Step into member DIEs
    if let Some((1, mut entry)) = entries.next_dfs()? {
        // Iterate over all the siblings until there's no more.
        loop {
            // Get the type of the member.
            if let Some(AttributeValue::Udata(count)) = entry.attr_value(gimli::DW_AT_count)? {
                lengths.push(count);
                continue;
            }

            let lower_bound = if let Some(AttributeValue::Udata(lower_bound)) =
                entry.attr_value(gimli::DW_AT_lower_bound)?
            {
                lower_bound
            } else {
                0
            };

            let upper_bound = if let Some(value) = entry.attr_value(gimli::DW_AT_upper_bound)? {
                // Size of array is upper bound + 1 - lower_bound
                match value {
                    AttributeValue::Data1(value) => value as u64 + 1,
                    AttributeValue::Data2(value) => value as u64 + 1,
                    AttributeValue::Data4(value) => value as u64 + 1,
                    AttributeValue::Data8(value) => value + 1,
                    AttributeValue::Udata(value) => value + 1,
                    // Zero sized array
                    AttributeValue::Sdata(-1) => 0,
                    _ => return Err(Error::BadAttribute),
                }
            } else {
                return Err(Error::Custom("Missing array upper bound!"));
            };

            lengths.push(upper_bound - lower_bound);

            // Get next sibling or break iteration.
            entry = if let Some(e) = entries.next_sibling()? {
                e
            } else {
                break;
            };
        }
    }

    Ok(Type::Array {
        ty: Box::new(ty),
        lengths,
    })
}

/// Parses a base type DIE
///
/// Output:
/// * Returns `Ok` if the base type DIE is successfully parsed.
/// * Returns `Err` if an error is encountered.
fn parse_base<R: Reader>(entry: &DebuggingInformationEntry<'_, '_, R>) -> Result<Type> {
    // TODO: use bit_size if byte_size not available?
    let byte_size = get_attribute!(entry, gimli::DW_AT_byte_size);
    let encoding = get_attribute!(entry, gimli::DW_AT_encoding);

    if let (AttributeValue::Udata(byte_size), AttributeValue::Encoding(encoding)) =
        (byte_size, encoding)
    {
        match (byte_size, encoding) {
            (1, gimli::DW_ATE_boolean) => Ok(Type::Bool),
            (1, gimli::DW_ATE_unsigned | gimli::DW_ATE_unsigned_char) => Ok(Type::U8),
            (2, gimli::DW_ATE_unsigned) => Ok(Type::U16),
            (4, gimli::DW_ATE_unsigned) => Ok(Type::U32),
            (8, gimli::DW_ATE_unsigned) => Ok(Type::U64),
            (1, gimli::DW_ATE_signed | gimli::DW_ATE_signed_char) => Ok(Type::I8),
            (2, gimli::DW_ATE_signed) => Ok(Type::I16),
            (4, gimli::DW_ATE_signed) => Ok(Type::I32),
            (8, gimli::DW_ATE_signed) => Ok(Type::I64),
            (4, gimli::DW_ATE_float) => Ok(Type::F32),
            (8, gimli::DW_ATE_float) => Ok(Type::F64),
            _ => Err(Error::UnsupportedBaseType(encoding, byte_size)),
        }
    } else {
        Err(Error::BadAttribute)
    }
}

/// Parses a pointer type DIE
///
/// Output:
/// * Returns `Ok` if the pointer type DIE is successfully parsed.
/// * Returns `Err` if an error is encountered.
fn parse_pointer<R: Reader>(entry: &DebuggingInformationEntry<'_, '_, R>) -> Result<Type> {
    let byte_size = get_attribute!(entry, gimli::DW_AT_byte_size);

    if let AttributeValue::Udata(byte_size) = byte_size {
        Ok(Type::Pointer(Box::new(match byte_size {
            1 => Type::U8,
            2 => Type::U16,
            4 => Type::U32,
            8 => Type::U64,
            _ => return Err(Error::UnsupportedPointerSize(byte_size)),
        })))
    } else {
        Err(Error::BadAttribute)
    }
}
