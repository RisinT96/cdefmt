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

        let raw = std::ffi::CStr::from_bytes_until_nul(&self.logs_section[id..])
            .map_err(|e| Error::NoNullTerm(id, e))?;
        let xml = raw.to_str().map_err(|e| Error::Utf8(id, e))?;
        let schema: SchemaVersion = quick_xml::de::from_str(xml)?;

        let mut metadata = match schema.version {
            1 => quick_xml::de::from_str::<Metadata>(xml),
            _ => return Err(Error::SchemaVersion(schema.version)),
        }?;

        metadata.id = id;

        Ok(metadata)
    }

    /// Returns an iterator over all of the log's metadata/type information.
    pub fn iter_logs(&self) -> MetadataIterator {
        MetadataIterator {
            parser: self,
            pos: 0,
        }
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

pub struct MetadataIterator<'data> {
    parser: &'data Parser<'data>,
    pos: usize,
}

impl<'data> Iterator for MetadataIterator<'data> {
    type Item = Result<(Metadata, Option<Type>)>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == self.parser.logs_section.len() {
            return None;
        }

        let metadata = match self.parser.get_log_metadata(self.pos) {
            Ok(m) => m,
            Err(e) => return Some(Err(e)),
        };

        let ty = match self.parser.get_log_args_type(&metadata) {
            Ok(t) => t,
            Err(e) => return Some(Err(e)),
        };

        self.pos = self.parser.logs_section[self.pos..]
            .iter()
            // Find end of current metadata
            .position(|c| *c == 0)
            .and_then(|null_delimiter| {
                self.parser.logs_section[self.pos + null_delimiter..]
                    .iter()
                    // Find start of next metadata
                    .position(|c| *c != 0)
                    .map(|new| self.pos + null_delimiter + new)
            })
            .unwrap_or_else(|| self.parser.logs_section.len());

        Some(Ok((metadata, ty)))
    }
}
