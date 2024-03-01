use gimli::{AttributeValue, DebuggingInformationEntry, UnitOffset};
use gimli::{Dwarf, EndianSlice, EntriesCursor, Reader, Unit};
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

#[derive(Debug)]
pub(crate) struct UncompressedDwarf<'data> {
    dwarf: Dwarf<Cow<'data, [u8]>>,
    endian: gimli::RunTimeEndian,
}

impl<'data> UncompressedDwarf<'data> {
    pub(crate) fn new<R: ReadRef<'data>>(file: &File<'data, R>) -> Result<Self> {
        let endian = if file.is_little_endian() {
            gimli::RunTimeEndian::Little
        } else {
            gimli::RunTimeEndian::Big
        };

        // Load a section and return as `Cow<[u8]>`.
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

    pub(crate) fn get_type(
        &'data self,
        compilation_unit: &str,
        type_name: &str,
    ) -> Result<Option<Type>> {
        let dwarf = self.borrow();
        let compilation_unit =
            if let Some(c) = Self::find_compilation_unit(&dwarf, compilation_unit)? {
                c
            } else {
                return Ok(None);
            };

        let unit_offset =
            if let Some(c) = Self::find_type_die(&dwarf, &compilation_unit, type_name)? {
                c
            } else {
                return Ok(None);
            };

        println!("Found type die!");
        Self::parse_type(&dwarf, &compilation_unit, unit_offset).map(move |t| Some(t))
    }

    fn borrow(&'data self) -> Dwarf<EndianSlice<'data, gimli::RunTimeEndian>> {
        // Borrow a `Cow<[u8]>` to create an `EndianSlice`.
        let borrow_section =
            |section: &'data Cow<'_, [u8]>| gimli::EndianSlice::new(section, self.endian);

        self.dwarf.borrow(&borrow_section)
    }

    fn find_compilation_unit<R: Reader>(dwarf: &Dwarf<R>, name: &str) -> Result<Option<Unit<R>>> {
        // Iterate over the compilation units.
        let mut iter = dwarf.units();
        while let Some(header) = iter.next()? {
            let unit = dwarf.unit(header)?;

            // Iterate over the Debugging Information Entries (DIEs) in the compilation units.
            let mut entries = unit.entries();
            while let Some((_, entry)) = entries.next_dfs()? {
                if entry.tag() != gimli::DW_TAG_compile_unit {
                    continue;
                }

                // Found compilation unit DIE, should usually be first in the file.

                // Find compilation unit name, check if it equals the provided name.
                let name_attribute = entry
                    .attr_value(gimli::DW_AT_name)?
                    .ok_or(Error::NoAttribute(gimli::DW_AT_name))?;
                let unit_name = dwarf.attr_string(&unit, name_attribute)?;
                let unit_name = unit_name.to_string()?;

                if name == unit_name {
                    return Ok(Some(unit));
                }
            }
        }

        Ok(None)
    }

    fn find_type_die<R: Reader>(
        dwarf: &Dwarf<R>,
        compilation_unit: &Unit<R>,
        type_name: &str,
    ) -> Result<Option<UnitOffset<R::Offset>>> {
        // Iterate over the Debugging Information Entries (DIEs) in the unit.
        let mut entries = compilation_unit.entries();
        while let Some((_, entry)) = entries.next_dfs()? {
            if let Some(name_attribute) = entry.attr_value(gimli::DW_AT_name)? {
                let name = dwarf.attr_string(&compilation_unit, name_attribute)?;
                let name = name.to_string()?;

                if name == type_name {
                    // Found our entry.
                    return Ok(Some(entry.offset()));
                }
            }
        }

        Ok(None)
    }

    fn parse_type<R: Reader>(
        dwarf: &Dwarf<R>,
        unit: &Unit<R>,
        start_offset: UnitOffset<R::Offset>,
    ) -> Result<Type> {
        let mut entries = unit.entries_at_offset(start_offset)?;

        if let Some((_, entry)) = entries.next_dfs()? {
            let tag = entry.tag();

            if let Some(name) = entry.attr_value(gimli::DW_AT_name)? {
                let name = dwarf.attr_string(unit, name)?;
                let name = name.to_string()?;
                println!("Tag: {tag}, name: {name}");
            } else {
                println!("Tag: {tag}");
            }

            // Parse known types
            match tag {
                gimli::DW_TAG_base_type => Self::parse_base(entry),
                gimli::DW_TAG_enumeration_type => {
                    Self::parse_enumeration(dwarf, unit, &mut entries)
                }
                gimli::DW_TAG_pointer_type => Self::parse_pointer(entry),
                gimli::DW_TAG_structure_type => Self::parse_structure(dwarf, unit, &mut entries),
                gimli::DW_TAG_const_type | gimli::DW_TAG_typedef => {
                    let type_ref = get_attribute!(entry, gimli::DW_AT_type);

                    if let AttributeValue::UnitRef(unit_ref) = type_ref {
                        Self::parse_type(dwarf, unit, unit_ref)
                    } else {
                        Err(Error::NoNullTerm)
                    }
                }
                _ => Err(Error::UnexpectedTag(tag)),
            }
        } else {
            Err(Error::NoNullTerm)
        }
    }

    /// Parses an enumeration DIE
    fn parse_enumeration<R: Reader>(
        dwarf: &Dwarf<R>,
        unit: &Unit<R>,
        entries: &mut EntriesCursor<'_, '_, R>,
    ) -> Result<Type> {
        println!("\tParsing enum!");
        let entry = entries.current().unwrap();

        let mut valid_values = BTreeMap::default();

        let ty = if let AttributeValue::UnitRef(unit_offset) =
            get_attribute!(entry, gimli::DW_AT_type)
        {
            Self::parse_type(dwarf, unit, unit_offset)?
        } else {
            return Err(Error::BadAttribute);
        };

        // Step into member DIEs
        if let Some((1, mut entry)) = entries.next_dfs()? {
            // Iterate over all the siblings until there's no more.
            loop {
                let name = get_attribute!(entry, gimli::DW_AT_name);
                let name = dwarf.attr_string(unit, name)?;
                let name = name.to_string()?.to_string();

                let value = match ty {
                    Type::I8 | Type::I16 | Type::I32 | Type::I64 => entry
                        .attr_value(gimli::DW_AT_const_value)?
                        .map(|o| o.sdata_value())
                        .flatten()
                        .unwrap_or(0)
                        as i128,
                    Type::U8 | Type::U16 | Type::U32 | Type::U64 => entry
                        .attr_value(gimli::DW_AT_const_value)?
                        .map(|o| o.udata_value())
                        .flatten()
                        .unwrap_or(0)
                        as i128,
                    _ => unreachable!("C enums must have integer types!"),
                };

                valid_values.insert(value, name);

                entry = if let Some(e) = entries.next_sibling()? {
                    e
                } else {
                    break;
                };
            }
        }

        Ok(Type::Enumeration {
            ty: Box::new(ty),
            valid_values,
        })
    }

    /// Parses a structure DIE
    fn parse_structure<R: Reader>(
        dwarf: &Dwarf<R>,
        unit: &Unit<R>,
        entries: &mut EntriesCursor<'_, '_, R>,
    ) -> Result<Type> {
        let entry = entries.current().unwrap();
        let name = entry
            .attr_value(gimli::DW_AT_name)?
            .ok_or(Error::NoAttribute(gimli::DW_AT_name))?;
        let name = dwarf.attr_string(unit, name)?;
        let name = name.to_string()?.to_string();

        let mut members = vec![];

        // Step into member DIEs
        if let Some((1, mut entry)) = entries.next_dfs()? {
            // Iterate over all the siblings until there's no more.
            loop {
                let name = get_attribute!(entry, gimli::DW_AT_name);
                let name = dwarf.attr_string(unit, name)?;
                let name = name.to_string()?.to_string();

                println!("Parsing member: {}", name);

                let offset = entry
                    .attr_value(gimli::DW_AT_data_member_location)?
                    .map(|o| o.udata_value())
                    .flatten()
                    .unwrap_or(0);

                if let AttributeValue::UnitRef(unit_offset) =
                    get_attribute!(entry, gimli::DW_AT_type)
                {
                    let ty = Self::parse_type(dwarf, unit, unit_offset)?;
                    members.push(StructureMember { name, ty, offset });
                } else {
                    return Err(Error::BadAttribute);
                }

                entry = if let Some(e) = entries.next_sibling()? {
                    e
                } else {
                    break;
                };
            }
        }

        Ok(Type::Structure { members, name })
    }

    /// Parses a base type DIE
    fn parse_base<R: Reader>(entry: &DebuggingInformationEntry<'_, '_, R>) -> Result<Type> {
        let encoding = get_attribute!(entry, gimli::DW_AT_encoding);
        let byte_size = get_attribute!(entry, gimli::DW_AT_byte_size);

        if let (AttributeValue::Encoding(encoding), AttributeValue::Udata(byte_size)) =
            (encoding, byte_size)
        {
            match (encoding, byte_size) {
                (gimli::DW_ATE_boolean, 1) => Ok(Type::Bool),
                (gimli::DW_ATE_unsigned, 1) => Ok(Type::U8),
                (gimli::DW_ATE_unsigned, 2) => Ok(Type::U16),
                (gimli::DW_ATE_unsigned, 4) => Ok(Type::U32),
                (gimli::DW_ATE_unsigned, 8) => Ok(Type::U64),
                (gimli::DW_ATE_signed, 1) => Ok(Type::I8),
                (gimli::DW_ATE_signed, 2) => Ok(Type::I16),
                (gimli::DW_ATE_signed, 4) => Ok(Type::I32),
                (gimli::DW_ATE_signed, 8) => Ok(Type::I64),
                (gimli::DW_ATE_float, 4) => Ok(Type::F32),
                (gimli::DW_ATE_float, 8) => Ok(Type::F64),
                _ => Err(Error::UnsupportedBaseType(encoding, byte_size)),
            }
        } else {
            Err(Error::NoNullTerm)
        }
    }

    /// Parses a pointer type DIE
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
            Err(Error::NoNullTerm)
        }
    }
}
