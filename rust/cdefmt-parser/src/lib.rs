pub mod dwarf;
pub mod log;

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
    #[error("Nullterminator is missing from log string")]
    NoNullTerm,
    #[error("Unable to find requested type ({0}).")]
    NoType(String),
    #[error("Provided log id [{0}] is larger than the '.cdefmt' section [{1}]")]
    OutOfBounds(usize, usize),
    #[error("Failed extract data from the '.cdefmt' section, error: {0}")]
    SectionData(#[from] object::Error),
    #[error("The log at id [{0}] is malformed, error: {1}")]
    Utf8(usize, std::str::Utf8Error),
}

pub type Result<T> = std::result::Result<T, Error>;
