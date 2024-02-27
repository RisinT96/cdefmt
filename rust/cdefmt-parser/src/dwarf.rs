use gimli::{
    DW_AT_name, DW_TAG_compile_unit, DebuggingInformationEntry, Dwarf, EndianSlice, Reader, Unit,
};
use gimli::{DW_TAG_structure_type, UnitOffset};
use object::{Object, ObjectSection, ReadRef};
use std::borrow::{self, Cow};

use crate::Error;
use crate::Result;

pub(crate) struct UncompressedDwarf<'data> {
    dwarf: Dwarf<Cow<'data, [u8]>>,
    endian: gimli::RunTimeEndian,
}

impl<'data> UncompressedDwarf<'data> {
    pub(crate) fn new<R: ReadRef<'data>>(data: R) -> Result<Self> {
        let file = object::File::parse(data)?;

        let endian = if file.is_little_endian() {
            gimli::RunTimeEndian::Little
        } else {
            gimli::RunTimeEndian::Big
        };

        // Load a section and return as `Cow<[u8]>`.
        let load_section = |id: gimli::SectionId| -> Result<std::borrow::Cow<[u8]>> {
            match file.section_by_name(id.name()) {
                Some(ref section) => Ok(section
                    .uncompressed_data()
                    .unwrap_or(std::borrow::Cow::Borrowed(&[][..]))),
                None => Ok(std::borrow::Cow::Borrowed(&[][..])),
            }
        };

        let dwarf = gimli::Dwarf::load(&load_section)?;

        // Load all of the sections.
        Ok(Self { dwarf, endian })
    }

    pub(crate) fn borrow(&'data self) -> Dwarf<EndianSlice<'data, gimli::RunTimeEndian>> {
        // Borrow a `Cow<[u8]>` to create an `EndianSlice`.
        let borrow_section =
            |section: &'data Cow<'_, [u8]>| gimli::EndianSlice::new(section, self.endian);

        self.dwarf.borrow(&borrow_section)
    }
}

pub fn find_compilation_unit<R: Reader>(dwarf: &Dwarf<R>, name: &str) -> Result<Unit<R>> {
    // Iterate over the compilation units.
    let mut iter = dwarf.units();
    while let Some(header) = iter.next()? {
        let unit = dwarf.unit(header)?;

        // Iterate over the Debugging Information Entries (DIEs) in the compilation units.
        let mut entries = unit.entries();
        while let Some((_, entry)) = entries.next_dfs()? {
            if entry.tag() != DW_TAG_compile_unit {
                continue;
            }

            // Found compilation unit DIE, should usually be first in the file.

            // Find compilation unit name, check if it equals the provided name.
            let name_attribute = entry
                .attr_value(DW_AT_name)?
                .ok_or(Error::NoAttribute(DW_AT_name))?;
            let unit_name = dwarf.attr_string(&unit, name_attribute)?;
            let unit_name = unit_name.to_string()?;

            if name == unit_name {
                return Ok(unit);
            }
        }
    }

    Err(Error::NoCompilationUnit(name.to_string()))
}

pub fn find_type<R: Reader>(
    dwarf: &Dwarf<R>,
    unit: &Unit<R>,
    type_name: &str,
) -> Result<UnitOffset<R::Offset>> {
    // Iterate over the Debugging Information Entries (DIEs) in the unit.
    let mut entries = unit.entries();
    while let Some((_, entry)) = entries.next_dfs()? {
        // We're looking for a struct
        if entry.tag() != DW_TAG_structure_type {
            continue;
        }

        // Check the struct's name.
        let name_attribute = entry
            .attr_value(DW_AT_name)?
            .ok_or(Error::NoAttribute(DW_AT_name))?;
        let name = dwarf.attr_string(&unit, name_attribute)?;
        let name = name.to_string()?;

        if name == type_name {
            // Found our entry.
            return Ok(entry.offset());
        }
    }

    Err(Error::NoType(type_name.to_string()))
}
