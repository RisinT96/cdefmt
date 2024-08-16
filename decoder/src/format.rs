use std::fmt::Alignment;

use lazy_regex::regex;

use crate::{Error, Result};

#[derive(Debug)]
pub(crate) struct Parameter<'s> {
    pub position: Option<ParameterPosition<'s>>,
    pub hint: DisplayHint,
}

impl<'s> Parameter<'s> {
    pub(crate) fn parse(param: &'s str) -> Result<Self> {
        let re = regex!(
            r"((?<position>\d+)|(?<named>[\d\w]+))?(:(?<align>[<\^>])?(?<sign>\+)?(?<alternate>#)?(?<zero_pad>0)?(?<width>\d+)?(\.(?<precision>\d+))?(?<type>[b\?exopeEX])?)?"
        );

        let captures = re.captures(param);

        if captures.is_none() {
            return Ok(Self {
                position: None,
                hint: DisplayHint {
                    align: None,
                    sign: false,
                    alternate: false,
                    zero_pad: false,
                    width: None,
                    precision: None,
                    ty: DisplayType::Display,
                },
            });
        }

        // Unwrap safety: returned earlier if captures is none.
        let captures = captures.unwrap();

        let position = if let Some(p) = captures.name("position") {
            Some(ParameterPosition::Positional(p.as_str().parse().unwrap()))
        } else {
            captures
                .name("named")
                .map(|n| ParameterPosition::Named(n.as_str()))
        };

        let align = captures.name("align").map(|a| match a.as_str() {
            "<" => Alignment::Left,
            "^" => Alignment::Center,
            ">" => Alignment::Right,
            _ => unreachable!("Regex should only capture valid values"),
        });

        let sign = captures.name("sign").is_some();
        let alternate = captures.name("alternate").is_some();
        let zero_pad = captures.name("zero_pad").is_some();
        let width = captures.name("width").map(|w| w.as_str().parse().unwrap());
        let precision = captures
            .name("precision")
            .map(|w| w.as_str().parse().unwrap());
        let ty = captures
            .name("type")
            .map(|ty| match ty.as_str() {
                "b" => DisplayType::Binary,
                "?" => DisplayType::Debug,
                "e" => DisplayType::LowerExp,
                "x" => DisplayType::LowerHex,
                "o" => DisplayType::Octal,
                "p" => DisplayType::Pointer,
                "E" => DisplayType::UpperExp,
                "X" => DisplayType::UpperHex,
                _ => unreachable!("Regex should only capture valid values"),
            })
            .unwrap_or(DisplayType::Display);

        Ok(Self {
            position,
            hint: DisplayHint {
                align,
                sign,
                alternate,
                zero_pad,
                width,
                precision,
                ty,
            },
        })
    }
}

#[derive(Debug)]
pub enum ParameterPosition<'s> {
    Positional(usize),
    Named(&'s str),
}

#[derive(Debug)]
pub enum DisplayType {
    Binary,
    Debug,
    Display,
    LowerExp,
    LowerHex,
    Octal,
    Pointer,
    UpperExp,
    UpperHex,
}

#[derive(Debug)]
pub struct DisplayHint {
    pub align: Option<std::fmt::Alignment>,
    pub sign: bool,
    pub alternate: bool,
    pub zero_pad: bool,
    pub width: Option<usize>,
    pub precision: Option<usize>,
    pub ty: DisplayType,
}

pub(crate) enum FormatStringFragment<'s> {
    Parameter(Parameter<'s>),
    Error(&'s str, Error),
    Escaped(char),
    Literal(&'s str),
}

pub(crate) struct FormatStringFragmentIterator<'s> {
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
                    let hint = &self.format_string[1..index];
                    self.format_string = self.format_string.get(index + 1..).unwrap_or("");
                    Some(FormatStringFragment::Parameter(
                        Parameter::parse(hint).unwrap(),
                    ))
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
