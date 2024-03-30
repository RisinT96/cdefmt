//! Log related logic.
//!
//! The logic contained within this file relates to using a log id to extract and parse the log's
//! information from the elf.

use lazy_regex::regex;

use core::fmt;

use regex::Captures;
use serde::Deserialize;
use serde_repr::Deserialize_repr;

use crate::var::Var;

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
    pub message: String,
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
}

impl std::fmt::Display for Log {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pattern = regex!("\\{\\}");

        let mut index = 0;

        let replacer = |_: &Captures| -> String {
            if self.args.is_empty() {
                // If we don't have any arguments, replace with empty string.
                String::new()
            } else {
                let value = self.args[index].format();
                index += 1;
                index %= self.args.len();
                value
            }
        };

        let message = pattern.replace_all(&self.metadata.message, replacer);

        write!(f, "{}", message)
    }
}
