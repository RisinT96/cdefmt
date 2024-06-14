//! Log related logic.
//!
//! The logic contained within this file relates to using a log id to extract and parse the log's
//! information from the elf.

use core::fmt;

use serde::Deserialize;
use serde_repr::Deserialize_repr;

use crate::{var::Var, Error};

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

enum FormatStringFragment<'s> {
    Argument,
    Error(&'s str, Error),
    Escaped(char),
    Literal(&'s str),
}

struct FormatStringFragmentIterator<'s> {
    format_string: &'s str,
}

impl<'s> FormatStringFragmentIterator<'s> {
    pub fn new(format_string: &'s str) -> Self {
        Self { format_string }
    }
}

impl<'s> Iterator for FormatStringFragmentIterator<'s> {
    type Item = FormatStringFragment<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        // Exhausted iterator
        if self.format_string.is_empty() {
            return None;
        }

        // Escaped opening braces.
        if self.format_string.starts_with("{{") {
            self.format_string = self.format_string.get(2..).unwrap_or("");
            return Some(FormatStringFragment::Escaped('{'));
        }

        // Non-escaped opening brace, try to find closing brace or error.
        if self.format_string.starts_with('{') {
            return match self.format_string.find('}') {
                Some(index) => {
                    self.format_string = self.format_string.get(index + 1..).unwrap_or("");
                    Some(FormatStringFragment::Argument)
                }
                None => {
                    let result = self.format_string;
                    self.format_string = "";
                    Some(FormatStringFragment::Error(
                        result,
                        Error::Custom("Malformed format string: missing closing brace"),
                    ))
                }
            };
        }

        // Regular literal.
        match self.format_string.find('{') {
            // Found opening brace, return substring until the brace and update self to start at brace.
            Some(index) => {
                let result = &self.format_string[..index];
                self.format_string = &self.format_string[index..];
                Some(FormatStringFragment::Literal(result))
            }
            // No opening brace, return entire format string.
            None => {
                let result = self.format_string;
                self.format_string = "";
                Some(FormatStringFragment::Literal(result))
            }
        }
    }
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
                FormatStringFragment::Argument => {
                    if !self.args.is_empty() {
                        let value = self.args[index].format();
                        index += 1;
                        index %= self.args.len();
                        write!(f, "{}", value)?
                    }
                }
                FormatStringFragment::Error(l, e) => write!(f, "{} ({})", l, e)?,
                FormatStringFragment::Escaped(c) => write!(f, "{}", c)?,
                FormatStringFragment::Literal(l) => write!(f, "{}", l)?,
            }
        }

        Ok(())
    }
}
