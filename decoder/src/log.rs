//! Log related logic.
//!
//! The logic contained within this file relates to using a log id to extract and parse the log's
//! information from the elf.

use cdefmt_parser::metadata::{Level, Metadata};
use rformat::{fmt::format::format_string, prelude::*};

use crate::{var::Var, Result};

#[derive(Clone, Debug)]
pub struct Log<'elf> {
    metadata: Metadata<'elf>,
    args: Vec<Var>,
}

impl<'elf> Log<'elf> {
    pub(crate) fn new(metadata: Metadata<'elf>, args: Vec<Var>) -> Self {
        Self { metadata, args }
    }

    pub fn get_level(&self) -> Level {
        self.metadata.level
    }

    pub fn get_file(&self) -> &str {
        self.metadata.file
    }

    pub fn get_line(&self) -> usize {
        self.metadata.line
    }

    pub fn get_args(&self) -> &[Var] {
        &self.args
    }
}

impl Log<'_> {
    pub fn to_string(&self) -> Result<String> {
        let params = self
            .args
            .iter()
            .zip(self.metadata.names.iter())
            .map(|(a, n)| rformat::fmt::format::Parameter {
                identifier: n,
                formattable: rformat::formattable::into_formattable!((*a)),
            })
            .collect::<Vec<_>>();

        Ok(format_string(self.metadata.fmt, &params)?)
    }
}
