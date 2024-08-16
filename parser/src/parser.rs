//! Contains logic related to finding logs in the elf and parsing them.

use object::{AddressSize, Object, ObjectSection, ReadRef};

use crate::{
    dwarf::Dwarf,
    metadata::{Metadata, SchemaVersion},
    r#type::Type,
    Error, Result,
};

/// Responsible for parsing logs from the elf.
pub struct Parser<'data> {
    logs_section: &'data [u8],
    build_id: &'data [u8],
    dwarf: Dwarf<'data>,
    address_size: AddressSize,
}

impl<'data> Parser<'data> {
    /// Creates a new Parser from elf data.
    pub fn new<R: ReadRef<'data>>(data: R) -> Result<Self> {
        let file = object::File::parse(data)?;
        let dwarf = Dwarf::new(&file)?;
        let build_id = file
            .build_id()?
            .ok_or(Error::Custom("Unable to find build ID in elf!"))?;

        let address_size = file.architecture().address_size().ok_or(Error::Custom(
            "Unsupported architecture, no address size information!",
        ))?;

        Ok(Parser {
            logs_section: file
                .section_by_name(".cdefmt")
                .ok_or(Error::MissingSection)?
                .data()?,
            build_id,
            dwarf,
            address_size,
        })
    }

    /// Returns a specific log's metadata.
    pub fn get_log_metadata(&self, id: usize) -> Result<Metadata> {
        if id >= self.logs_section.len() {
            return Err(Error::OutOfBounds(id, self.logs_section.len()));
        }

        let json = &self.logs_section[id..]
            .split(|b| *b == 0)
            .next()
            .ok_or(Error::NoNullTerm)?;
        let json = std::str::from_utf8(json).map_err(|e| Error::Utf8(id, e))?;
        let schema: SchemaVersion = serde_json::from_str(json)?;

        let mut metadata = match schema.version {
            1 => serde_json::from_str::<Metadata>(json),
            _ => return Err(Error::SchemaVersion(schema.version)),
        }?;

        metadata.id = id;

        Ok(metadata)
    }

    /// Returns the type of the log's arguments.
    /// Return:
    /// * Ok(Some(_)) => The type of the arguments.
    /// * Ok(None)    => Unable to find the type in the elf's dwarf section.
    /// * Err(_)      => Encountered some error while parsing the dwarf.
    pub fn get_log_args_type(&self, metadata: &Metadata) -> Result<Option<Type>> {
        let type_name = format!("cdefmt_log_args_t{}", metadata.counter);
        self.dwarf.get_type(&metadata.file, &type_name)
    }

    pub fn build_id(&self) -> &'data [u8] {
        self.build_id
    }

    pub fn address_size(&self) -> AddressSize {
        self.address_size
    }

    pub fn endian(&self) -> gimli::RunTimeEndian {
        self.dwarf.endian()
    }
}
