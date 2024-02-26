use cdefmt_parser::log::LogParser;

/// Takes path to elf as argument, and parses logs whose IDs are read from stdin.
/// Example usage:
/// <path to example-stdout> | cargo run examples/stdin <path to example-stdout>
fn main() -> std::result::Result<(), String> {
    let path = std::env::args().nth(2).ok_or("Y U NO GIVE PATH?")?;
    let file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mmap = unsafe { memmap2::Mmap::map(&file) }.map_err(|e| e.to_string())?;
    let object = object::File::parse(&*mmap).map_err(|e| e.to_string())?;

    let mut logger = LogParser::new(&object).map_err(|e| e.to_string())?;

    let stdin = std::io::stdin();
    let mut buff = Default::default();

    while let Ok(s) = stdin.read_line(&mut buff) {
        if s == 0 {
            break;
        }

        let log_id = buff.trim().parse::<usize>().map_err(|e| e.to_string())?;

        let log = logger.get_log(log_id).map_err(|e| e.to_string())?;
        println!("[{:#010x}] {:?}", log_id, log);

        buff.clear();
    }

    Ok(())
}

// fn dump_file(object: &object::File, endian: gimli::RunTimeEndian) -> Result<(), gimli::Error> {
// use gimli::{DW_AT_name, DW_TAG_compile_unit};
// use object::{Object, ObjectSection};
// use std::{borrow, env, fs};

//     let endian = if object.is_little_endian() {
//         gimli::RunTimeEndian::Little
//     } else {
//         gimli::RunTimeEndian::Big
//     };

//     // Load a section and return as `Cow<[u8]>`.
//     let load_section = |id: gimli::SectionId| -> Result<borrow::Cow<[u8]>, gimli::Error> {
//         match object.section_by_name(id.name()) {
//             Some(ref section) => Ok(section
//                 .uncompressed_data()
//                 .unwrap_or(borrow::Cow::Borrowed(&[][..]))),
//             None => Ok(borrow::Cow::Borrowed(&[][..])),
//         }
//     };

//     // Load all of the sections.
//     let dwarf_cow = gimli::Dwarf::load(&load_section)?;

//     // Borrow a `Cow<[u8]>` to create an `EndianSlice`.
//     let borrow_section: &dyn for<'a> Fn(
//         &'a borrow::Cow<[u8]>,
//     ) -> gimli::EndianSlice<'a, gimli::RunTimeEndian> =
//         &|section| gimli::EndianSlice::new(section, endian);

//     // Create `EndianSlice`s for all of the sections.
//     let dwarf = dwarf_cow.borrow(&borrow_section);

//     // Iterate over the compilation units.
//     let mut iter = dwarf.units();
//     while let Some(header) = iter.next()? {
//         println!(
//             "Unit at <.debug_info+0x{:x}>",
//             header.offset().as_debug_info_offset().unwrap().0
//         );
//         let unit = dwarf.unit(header)?;

//         // Iterate over the Debugging Information Entries (DIEs) in the unit.
//         let mut depth = 0;
//         let mut entries = unit.entries();
//         while let Some((delta_depth, entry)) = entries.next_dfs()? {
//             depth += delta_depth;

//             if entry.tag() != DW_TAG_compile_unit {
//                 continue;
//             }

//             println!("<{}><{:x}> {}", depth, entry.offset().0, entry.tag());

//             // Iterate over the attributes in the DIE.
//             let mut attrs = entry.attrs();
//             while let Some(attr) = attrs.next()? {
//                 println!("   {}: {:?}", attr.name(), attr.value());
//             }

//             let name = entry.attr_value(DW_AT_name)?.unwrap();
//             let name = dwarf.attr_string(&unit, name)?.to_string()?;
//             println!("Name: {:?}", name);
//         }
//     }
//     Ok(())
// }
