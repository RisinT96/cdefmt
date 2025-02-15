use gimli::{DwAte, DwTag, SectionId};

mod dwarf;

pub mod metadata;
pub mod parser;
pub mod r#type;

pub use parser::Parser;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Gimli error: {0}")]
    Gimli(#[from] gimli::Error),
    #[error("Json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("The provided elf is missing the '.cdefmt' section.")]
    MissingSection,
    #[error("DIE is missing attribute {0}")]
    NoAttribute(gimli::DwAt),
    #[error("Unable to find requested compilation unit ({0}).")]
    NoCompilationUnit(String),
    #[error("The log at id [{0}] has no null terminator!, error: {1}")]
    NoNullTerm(usize, std::ffi::FromBytesUntilNulError),
    #[error("The elf is missing the following section: {0:?}")]
    NoSection(SectionId),
    #[error("Unable to find requested type ({0}).")]
    NoType(String),
    #[error("Provided log id [{0}] is larger than the '.cdefmt' section [{1}]")]
    OutOfBounds(usize, usize),
    #[error("Failed extract data from the '.cdefmt' section, error: {0}")]
    SectionData(#[from] object::Error),
    #[error("The log at id [{0}] is malformed, error: {1}")]
    Utf8(usize, std::str::Utf8Error),
    #[error("Encountered an unsupported base type, encoding: {0}, size: {1}")]
    UnsupportedBaseType(DwAte, u64),
    #[error("Encountered an unsupported pointer size: {0}")]
    UnsupportedPointerSize(u64),
    #[error("Encountered an unexpected tag: {0}")]
    UnexpectedTag(DwTag),
    #[error("Encountered an attribute with bad type")]
    BadAttribute,
    #[error("There is no DIE at the given offset: {0}")]
    NoDIE(u64),
    #[error("Unsupported schema version: {0}")]
    SchemaVersion(u32),
    #[error("{0}")]
    Custom(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;
