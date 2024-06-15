//! Log related logic.
//!
//! The logic contained within this file relates to using a log id to extract and parse the log's
//! information from the elf.

use core::fmt;

use serde::Deserialize;
use serde_repr::Deserialize_repr;

use crate::{
    format::{FormatStringFragment, FormatStringFragmentIterator, ParameterPosition},
    var::Var,
};

#[derive(Clone, Copy, Debug, Deserialize_repr)]
#[repr(u8)]
pub enum Level {
    Error,
    Warning,
    Info,
    Debug,
    Verbose,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(&format!("{:?}", self))
    }
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct Schema {
    pub schema: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct MetadataV1 {
    #[serde(skip)]
    pub id: usize,
    pub counter: usize,
    pub level: Level,
    pub file: String,
    pub line: usize,
    #[serde(rename = "message")]
    pub format_string: String,
}

#[derive(Clone, Debug)]
pub struct Log {
    metadata: MetadataV1,
    args: Vec<Var>,
}

impl Log {
    pub(crate) fn new(metadata: MetadataV1, args: Vec<Var>) -> Self {
        Self { metadata, args }
    }

    pub fn get_level(&self) -> Level {
        self.metadata.level
    }

    pub fn get_file(&self) -> &str {
        &self.metadata.file
    }

    pub fn get_line(&self) -> usize {
        self.metadata.line
    }

    pub fn get_args(&self) -> &[Var] {
        &self.args
    }

    fn get_format_fragments(&self) -> FormatStringFragmentIterator {
        FormatStringFragmentIterator::new(&self.metadata.format_string)
    }
}

impl std::fmt::Display for Log {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut index = 0;

        for fragment in self.get_format_fragments() {
            match fragment {
                FormatStringFragment::Parameter(parameter) => {
                    if !self.args.is_empty() {
                        let var = match parameter.position {
                            Some(ParameterPosition::Positional(position)) => {
                                &self.args[position % self.args.len()]
                            }
                            Some(ParameterPosition::Named(_)) => todo!(),
                            None => {
                                let res = &self.args[index];
                                index += 1;
                                index %= self.args.len();
                                res
                            }
                        };
                        write!(f, "{}", var.format(&parameter.hint))?
                    }
                }
                FormatStringFragment::Error(literal, error) => {
                    write!(f, "{} ({})", literal, error)?
                }
                FormatStringFragment::Escaped(character) => write!(f, "{}", character)?,
                FormatStringFragment::Literal(literal) => write!(f, "{}", literal)?,
            }
        }

        Ok(())
    }
}
