//! Log related logic.
//!
//! The logic contained within this file relates to using a log id to extract and parse the log's
//! information from the elf.

use std::collections::HashMap;

use object::{Object, ObjectSection, ReadRef};
use serde::Deserialize;
use serde_repr::Deserialize_repr;

#[derive(Debug, thiserror::Error)]
pub enum Error<'str> {
    #[error("The provided elf is missing the '.cdefmt' section.")]
    MissingSection,
    #[error("Failed extract data from the '.cdefmt' section, error: {0}")]
    SectionData(#[from] object::Error),
    #[error("Provided log id [{0}] is larger than the '.cdefmt' section [{1}]")]
    OutOfBounds(usize, usize),
    #[error("The log at id [{0}] is malformed, error: {1}")]
    Utf8(usize, std::str::Utf8Error),
    #[error("The log [{0}] is malformed: {1}")]
    Json(&'str str, serde_json::Error),
    #[error("Nullterminator is missing from log string")]
    NoNullTerm,
}

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
    data: &'data [u8],
}

impl<'data> LogParser<'data> {
    pub fn new<R: ReadRef<'data>>(data: R) -> Result<LogParser<'data>, Error<'static>> {
        let file = object::File::parse(data)?;
        Ok(LogParser {
            cache: Default::default(),
            data: file
                .section_by_name(".cdefmt")
                .ok_or(Error::MissingSection)?
                .data()?,
        })
    }

    pub fn get_log(&mut self, log_id: usize) -> Result<Log, Error> {
        if log_id >= self.data.len() {
            return Err(Error::OutOfBounds(log_id, self.data.len()));
        }

        if self.cache.contains_key(&log_id) {
            return Ok(self.cache[&log_id]);
        }

        let log = &self.data[log_id..]
            .split(|b| *b == 0)
            .next()
            .ok_or(Error::NoNullTerm)?;
        let log = std::str::from_utf8(log).map_err(|e| Error::Utf8(log_id, e))?;
        let log = serde_json::from_str(log).map_err(|e| Error::Json(log, e))?;
        self.cache.insert(log_id, log);

        Ok(log)
    }
}
