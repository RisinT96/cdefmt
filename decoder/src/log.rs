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
pub struct Log {
    metadata: Metadata,
    args: Option<Vec<Var>>,
}

impl Log {
    pub(crate) fn new(metadata: Metadata, args: Option<Vec<Var>>) -> Self {
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

    pub fn get_args(&self) -> Option<&[Var]> {
        self.args.as_deref()
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
                    if self.args.is_none() {
                        return write!(f, "{{}}");
                    }

                    let args = self.args.as_ref().unwrap();

                    if args.is_empty() {
                        return write!(f, "{{}}");
                    }

                    let var = match parameter.position {
                        Some(ParameterPosition::Positional(position)) => {
                            &args[position % args.len()]
                        }
                        Some(ParameterPosition::Named(_)) => todo!(),
                        None => {
                            let res = &args[index];
                            index += 1;
                            index %= args.len();
                            res
                        }
                    };
                    write!(f, "{}", var.format(&parameter.hint))?
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
