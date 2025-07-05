//! Log related logic.
//!
//! The logic contained within this file relates to using a log id to extract and parse the log's
//! information from the elf.

use core::fmt;

use cdefmt_parser::metadata::{Level, Metadata};

use crate::{
    format::{FormatStringFragment, FormatStringFragmentIterator, ParameterPosition},
    var::Var,
};

#[derive(Clone, Debug)]
pub struct Log<'elf> {
    metadata: Metadata<'elf>,
    args: Option<Vec<Var>>,
}

impl<'elf> Log<'elf> {
    pub(crate) fn new(metadata: Metadata<'elf>, args: Option<Vec<Var>>) -> Self {
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

    pub fn get_args(&self) -> Option<&[Var]> {
        self.args.as_deref()
    }

    fn get_format_fragments(&self) -> FormatStringFragmentIterator {
        FormatStringFragmentIterator::new(self.metadata.fmt)
    }
}

impl std::fmt::Display for Log<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut index = 0;

        let args = self.args.as_deref().unwrap_or(&[]);

        for fragment in self.get_format_fragments() {
            match fragment {
                FormatStringFragment::Parameter(parameter) => {
                    let var = match parameter.position {
                        Some(ParameterPosition::Positional(position)) => {
                            if position >= args.len() {
                                write!(f, "{{No positional parameter at index {position}}}")?;
                                continue;
                            }

                            &args[position]
                        }
                        Some(ParameterPosition::Named(name)) => {
                            if let Some(pos) = self.metadata.names.iter().position(|&n| n == name) {
                                &args[pos]
                            } else {
                                write!(f, "{{No named parameter '{name}'}}")?;
                                continue;
                            }
                        }
                        None => {
                            if index >= args.len() {
                                write!(f, "{{No parameter at index {index}}}")?;
                                continue;
                            }

                            let res = &args[index];
                            index += 1;
                            res
                        }
                    };
                    write!(f, "{}", var.format(&parameter.hint))?
                }
                FormatStringFragment::Error(literal, error) => write!(f, "{literal} ({error})")?,
                FormatStringFragment::Escaped(character) => write!(f, "{}", character)?,
                FormatStringFragment::Literal(literal) => write!(f, "{}", literal)?,
            }
        }

        Ok(())
    }
}
