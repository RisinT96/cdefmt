use gimli::{DW_AT_name, DW_TAG_compile_unit, Dwarf, Reader, Result, Unit};
use object::{Object, ObjectSection};
use std::borrow;

pub fn find_compilation_unit<R: Reader>(dwarf: Dwarf<R>, name: &str) -> Result<Unit<R>> {
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
                .ok_or(gimli::Error::UnexpectedNull)?;
            let unit_name = dwarf.attr_string(&unit, name_attribute)?;
            let unit_name = unit_name.to_string()?;

            if name == unit_name {
                return Ok(unit);
            }
        }
    }

    Result::Err(gimli::Error::AbbreviationTagZero)
}
pub fn dump_file(object: &object::File) -> Result<()> {
    let endian = if object.is_little_endian() {
        gimli::RunTimeEndian::Little
    } else {
        gimli::RunTimeEndian::Big
    };

    // Load a section and return as `Cow<[u8]>`.
    let load_section = |id: gimli::SectionId| -> Result<borrow::Cow<[u8]>> {
        match object.section_by_name(id.name()) {
            Some(ref section) => Ok(section
                .uncompressed_data()
                .unwrap_or(borrow::Cow::Borrowed(&[][..]))),
            None => Ok(borrow::Cow::Borrowed(&[][..])),
        }
    };

    // Load all of the sections.
    let dwarf_cow = gimli::Dwarf::load(&load_section)?;

    // Borrow a `Cow<[u8]>` to create an `EndianSlice`.
    let borrow_section: &dyn for<'a> Fn(
        &'a borrow::Cow<[u8]>,
    ) -> gimli::EndianSlice<'a, gimli::RunTimeEndian> =
        &|section| gimli::EndianSlice::new(section, endian);

    // Create `EndianSlice`s for all of the sections.
    let dwarf = dwarf_cow.borrow(&borrow_section);

    let unit = find_compilation_unit(dwarf, "/root/git/cdefmt/c/examples/stdout/main.c")?;
    println!("Found unit!");

    Ok(())
}
