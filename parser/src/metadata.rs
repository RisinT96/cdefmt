//! Representation of log metadata extracted from the target elf's .cdefmt section.

use core::fmt;

use serde::Deserialize;
use serde_repr::Deserialize_repr;

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
pub(crate) struct SchemaVersion {
    pub version: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Metadata {
    #[serde(skip)]
    pub id: usize,
    pub counter: usize,
    pub level: Level,
    pub file: String,
    pub line: usize,
    #[serde(rename = "message")]
    pub format_string: String,
}
