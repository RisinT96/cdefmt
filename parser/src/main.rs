//! A simple example of parsing `.debug_info`.
mod log;

use log::{Log, LogParser};

use gimli::{DW_AT_name, DW_TAG_compile_unit};
use object::{Object, ObjectSection};
use std::{borrow, env, fs, io::Read};

fn main() -> serde_json::Result<()> {
    for path in env::args().skip(1) {
        let file = fs::File::open(&path).unwrap();
        let mmap = unsafe { memmap2::Mmap::map(&file).unwrap() };
        let object = object::File::parse(&*mmap).unwrap();
        let endian = if object.is_little_endian() {
            gimli::RunTimeEndian::Little
        } else {
            gimli::RunTimeEndian::Big
        };

        let mut logger = LogParser::new(&object).unwrap();

        let stdin = std::io::stdin();
        let mut buff = Default::default();

        while let Ok(s) = stdin.read_line(&mut buff) {
            if s == 0 {
                break;
            }

            let log_id = buff.trim();
            println!("Parsing: [{}]", log_id);
            let log_id = usize::from_str_radix(&log_id, 10).unwrap();

            let log = logger.get_log(log_id);
            println!("Log: {:?}", log);

            buff.clear();
        }
    }

    Ok(())
}

fn dump_file(object: &object::File, endian: gimli::RunTimeEndian) -> Result<(), gimli::Error> {
    // Load a section and return as `Cow<[u8]>`.
    let load_section = |id: gimli::SectionId| -> Result<borrow::Cow<[u8]>, gimli::Error> {
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

    // Iterate over the compilation units.
    let mut iter = dwarf.units();
    while let Some(header) = iter.next()? {
        println!(
            "Unit at <.debug_info+0x{:x}>",
            header.offset().as_debug_info_offset().unwrap().0
        );
        let unit = dwarf.unit(header)?;

        // Iterate over the Debugging Information Entries (DIEs) in the unit.
        let mut depth = 0;
        let mut entries = unit.entries();
        while let Some((delta_depth, entry)) = entries.next_dfs()? {
            depth += delta_depth;

            if entry.tag() != DW_TAG_compile_unit {
                continue;
            }

            println!("<{}><{:x}> {}", depth, entry.offset().0, entry.tag());

            // Iterate over the attributes in the DIE.
            let mut attrs = entry.attrs();
            while let Some(attr) = attrs.next()? {
                println!("   {}: {:?}", attr.name(), attr.value());
            }

            let name = entry.attr_value(DW_AT_name)?.unwrap();
            let name = dwarf.attr_string(&unit, name)?.to_string()?;
            println!("Name: {:?}", name);
        }
    }
    Ok(())
}
