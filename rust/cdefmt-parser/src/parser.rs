//! Contains logic related to finding logs in the elf and parsing them.

use std::collections::HashMap;

use gimli::Reader;
use object::{AddressSize, Object, ObjectSection, ReadRef};

use crate::{
    dwarf::Dwarf,
    log::{Log, LogInfo},
    r#type::Type,
    var::Var,
    Error, Result,
};

/// Responsible for parsing logs from the elf.
pub struct Parser<'data> {
    log_cache: HashMap<usize, Log>,
    logs_section: &'data [u8],
    dwarf: Dwarf<'data>,
    address_size: AddressSize,
}

impl<'data> Parser<'data> {
    /// Creates a new Parser from elf data.
    pub fn new<R: ReadRef<'data>>(data: R) -> Result<Self> {
        let file = object::File::parse(data)?;
        let dwarf = Dwarf::new(&file)?;

        let address_size = file.architecture().address_size().unwrap();
        Ok(Parser {
            log_cache: Default::default(),
            logs_section: file
                .section_by_name(".cdefmt")
                .ok_or(Error::MissingSection)?
                .data()?,
            dwarf,
            address_size,
        })
    }

    // Parses a log and returns it.
    pub fn parse_log(&mut self, data: &[u8]) -> Result<Log> {
        let mut data = gimli::EndianSlice::new(data, self.dwarf.endian());
        // TODO: Make safer, maybe switch to u64 everywhere.
        let log_id = data.read_address(self.address_size.bytes())? as usize;

        if log_id >= self.logs_section.len() {
            return Err(Error::OutOfBounds(log_id, self.logs_section.len()));
        }

        if self.log_cache.contains_key(&log_id) {
            return Ok(self.log_cache[&log_id].clone());
        }

        let log_info = self.get_log_info(log_id)?;
        let args = self.parse_log_args(&log_info, data)?;
        let log = Log::new(log_info, args);

        self.log_cache.insert(log_id, log.clone());

        Ok(log)
    }

    // Parses the log's static information.
    fn get_log_info(&self, log_id: usize) -> Result<LogInfo> {
        let log = &self.logs_section[log_id..]
            .split(|b| *b == 0)
            .next()
            .ok_or(Error::NoNullTerm)?;
        let log = std::str::from_utf8(log).map_err(|e| Error::Utf8(log_id, e))?;
        let log: LogInfo = serde_json::from_str(log)?;

        Ok(log)
    }

    // Parses the log's arguments.
    fn parse_log_args<R: Reader>(&self, log_info: &LogInfo, mut data: R) -> Result<Vec<Var>> {
        let type_name = format!("cdefmt_log_args_t{}", log_info.counter);
        let ty = self.dwarf.get_type(&log_info.file, &type_name)?.unwrap();

        let members = if let Type::Structure(mut members) = ty {
            // Due to the way the log is constructed in the c code, the first argument is always the
            // log id.
            assert!(members[0].name == "log_id");
            members.remove(0);
            members
        } else {
            return Err(Error::Custom("The log's args aren't a structure!"));
        };

        // Parse the raw data into `Var` representation.
        members
            .iter()
            .map(|m| Ok(Var::parse(&m.ty, &mut data)?.0))
            .collect()
    }
}
