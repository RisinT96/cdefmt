//! Representation of log metadata extracted from the target elf's .cdefmt section.

use core::{fmt, str};

use gimli::{EndianSlice, Reader, RunTimeEndian};

use crate::{Error, Result};

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Level {
    Error = 0,
    Warning = 1,
    Info = 2,
    Debug = 3,
    Verbose = 4,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(&format!("{:?}", self))
    }
}

#[derive(Clone, Debug)]
pub struct Metadata<'elf> {
    pub id: usize,
    pub counter: u32,
    pub line: usize,
    pub file: &'elf str,
    pub fmt: &'elf str,
    pub names: Vec<&'elf str>,
    pub level: Level,
}

fn parse_metadata_impl<'elf>(
    id: usize,
    endian_slice: &mut EndianSlice<'elf, RunTimeEndian>,
) -> Result<(Metadata<'elf>, usize)> {
    let mut offset = 0;

    let version = endian_slice.read_u32()?;
    offset += 4;

    if version != 1 {
        return Err(Error::SchemaVersion(version).into());
    }

    let counter = endian_slice.read_u32()?;
    let line = endian_slice.read_u32()? as usize;
    offset += 4 * 2;

    let file_len = endian_slice.read_u32()? as usize;
    let fmt_len = endian_slice.read_u32()? as usize;
    let names_len = endian_slice.read_u32()? as usize;
    offset += 4 * 3;

    let level = endian_slice.read_u8()?;
    offset += 1;

    let file = endian_slice.split(file_len)?;
    let file = str::from_utf8(&file.slice()[..file_len - 1]).map_err(|e| Error::Utf8(id, e))?;
    offset += file_len;

    let fmt = endian_slice.split(fmt_len)?;
    let fmt = str::from_utf8(&fmt.slice()[..fmt_len - 1]).map_err(|e| Error::Utf8(id, e))?;
    offset += fmt_len;

    let names = (0..names_len)
        .map(|_| {
            let name_len = endian_slice.read_u32()? as usize;
            offset += 4;
            let name = endian_slice.split(name_len)?;
            offset += name_len;
            str::from_utf8(&name.slice()[..name_len - 1]).map_err(|e| Error::Utf8(id, e).into())
        })
        .collect::<Result<Vec<_>>>()?;

    Ok((
        Metadata {
            id,
            counter,
            line,
            file,
            fmt,
            names,
            level: unsafe { std::mem::transmute::<u8, Level>(level) },
        },
        offset,
    ))
}

pub(crate) fn parse_metadata(
    cdefmt_section: &[u8],
    id: usize,
    endian: RunTimeEndian,
) -> Result<Metadata<'_>> {
    let mut endian_slice = EndianSlice::new(cdefmt_section, endian);

    endian_slice
        .skip(id)
        .map_err(|_| Error::OutOfBounds(id, cdefmt_section.len()))?;

    Ok(parse_metadata_impl(id, &mut endian_slice)?.0)
}
