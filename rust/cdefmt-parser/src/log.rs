//! Log related logic.
//!
//! The logic contained within this file relates to using a log id to extract and parse the log's
//! information from the elf.

use core::fmt;
use std::collections::HashMap;

use object::{Object, ObjectSection, ReadRef};
use serde::Deserialize;
use serde_repr::Deserialize_repr;

use crate::{dwarf::UncompressedDwarf, Error, Result};

#[derive(Clone, Copy, Debug, Deserialize_repr)]
#[repr(u8)]
pub enum Level {
    Error,
    Warning,
    Info,
    Debug,
    Verbose,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Log<'str> {
    counter: usize,
    level: Level,
    file: &'str str,
    line: usize,
    message: &'str str,

}

pub struct LogParser<'data> {
    cache: HashMap<usize, Log<'data>>,
    logs_section: &'data [u8],
    dwarf: UncompressedDwarf<'data>,
}

impl<'data> LogParser<'data> {
    pub fn new<R: ReadRef<'data> + fmt::Debug>(data: R) -> Result<Self> {
        // The clone is necessary as the generation of the file  til
        // The clone here should only clone the reader, not the data.
        let file = object::File::parse(data)?;
        let dwarf = UncompressedDwarf::new(&file)?;
        Ok(LogParser {
            cache: Default::default(),
            logs_section: file
                .section_by_name(".cdefmt")
                .ok_or(Error::MissingSection)?
                .data()?,
            dwarf,
        })
    }

    pub fn get_log(&mut self, log_id: usize) -> Result<Log> {
        if log_id >= self.logs_section.len() {
            return Err(Error::OutOfBounds(log_id, self.logs_section.len()));
        }

        if self.cache.contains_key(&log_id) {
            return Ok(self.cache[&log_id]);
        }

        let log = &self.logs_section[log_id..]
            .split(|b| *b == 0)
            .next()
            .ok_or(Error::NoNullTerm)?;
        let log = std::str::from_utf8(log).map_err(|e| Error::Utf8(log_id, e))?;
        let log = serde_json::from_str(log)?;
        self.cache.insert(log_id, log);

        let type_name = format!("cdefmt_log_args_t{}", log.counter);

        let res = self.dwarf.get_type(log.file, &type_name);
        println!("{:?}", res);

        Ok(log)
    }
}
