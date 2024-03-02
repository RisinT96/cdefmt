//! Log related logic.
//!
//! The logic contained within this file relates to using a log id to extract and parse the log's
//! information from the elf.

use lazy_regex::regex;

use core::fmt;
use std::{collections::HashMap, fmt::Debug};

use gimli::Reader;
use object::{AddressSize, Object, ObjectSection, ReadRef};
use regex::{Captures, Regex};
use serde::Deserialize;
use serde_repr::Deserialize_repr;

use crate::{dwarf::UncompressedDwarf, r#type::Type, var::Var, Error, Result};

#[derive(Clone, Copy, Debug, Deserialize_repr)]
#[repr(u8)]
pub enum Level {
    Error,
    Warning,
    Info,
    Debug,
    Verbose,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Log {
    counter: usize,
    level: Level,
    file: String,
    line: usize,
    message: String,
}

#[derive(Clone, Debug)]
pub struct DataLog {
    log: Log,
    args: Vec<Var>,
}

pub struct LogParser<'data> {
    cache: HashMap<usize, Log>,
    logs_section: &'data [u8],
    dwarf: UncompressedDwarf<'data>,
    address_size: AddressSize,
}

impl<'data> LogParser<'data> {
    pub fn new<R: ReadRef<'data>>(data: R) -> Result<Self> {
        // The clone is necessary as the generation of the file  til
        // The clone here should only clone the reader, not the data.
        let file = object::File::parse(data)?;
        let dwarf = UncompressedDwarf::new(&file)?;

        let address_size = file.architecture().address_size().unwrap();
        Ok(LogParser {
            cache: Default::default(),
            logs_section: file
                .section_by_name(".cdefmt")
                .ok_or(Error::MissingSection)?
                .data()?,
            dwarf,
            address_size,
        })
    }

    pub fn parse_log(&mut self, data: &[u8]) -> Result<DataLog> {
        let mut data = gimli::EndianSlice::new(data, self.dwarf.endian);
        // TODO: Make safer, maybe switch to u64 everywhere.
        let log_id = data.read_address(self.address_size.bytes())? as usize;
        let log = self.get_log(log_id)?;
        let log_args = self.parse_log_args(&log, data)?;

        Ok(DataLog {
            log: log,
            args: log_args,
        })
    }

    fn get_log(&mut self, log_id: usize) -> Result<Log> {
        if log_id >= self.logs_section.len() {
            return Err(Error::OutOfBounds(log_id, self.logs_section.len()));
        }

        if self.cache.contains_key(&log_id) {
            return Ok(self.cache[&log_id].clone());
        }

        let log = &self.logs_section[log_id..]
            .split(|b| *b == 0)
            .next()
            .ok_or(Error::NoNullTerm)?;
        let log = std::str::from_utf8(log).map_err(|e| Error::Utf8(log_id, e))?;
        let log: Log = serde_json::from_str(log)?;
        self.cache.insert(log_id, log.clone());

        Ok(log)
    }

    fn parse_log_args<R: Reader>(&self, log: &Log, mut data: R) -> Result<Vec<Var>> {
        let type_name = format!("cdefmt_log_args_t{}", log.counter);
        let ty = self.dwarf.get_type(log.file.as_str(), &type_name)?.unwrap();
        let members = if let Type::Structure { name, mut members } = ty {
            // The first member is actually the log ID, we already parsed it earlier.
            members.remove(0);
            members
        } else {
            return Err(Error::Custom("Something's fucked!"));
        };

        members
            .iter()
            .map(|m| Ok(Var::parse(&m.ty, &mut data)?.0))
            .collect()
    }
}

impl std::fmt::Display for DataLog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pattern = regex!("\\{\\}");

        let mut index = 0;

        let replacer = |_: &Captures| -> String {
            let res = format!("{:?}", self.args[index]);

            index += 1;
            index %= self.args.len();

            res
        };

        let message = pattern.replace_all(&self.log.message, replacer);

        // let message = pattern.replace_all(&self.log.message, |m| -> Result<String> {
        // });
        // self.log.message.re
        write!(f, "{}", message)
    }
}
