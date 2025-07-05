//! Contains logic related to finding logs in the elf and parsing them.

use object::{AddressSize, Object, ObjectSection, ObjectSymbol, ReadRef};

use crate::{
    dwarf::Dwarf,
    metadata::{parse_metadata, Metadata},
    r#type::Type,
    Error, Result,
};

/// Responsible for parsing logs from the elf.
pub struct Parser<'elf> {
    logs_section: &'elf [u8],
    build_id: &'elf [u8],
    dwarf: Dwarf<'elf>,
    address_size: AddressSize,
    metadata_addresses: Vec<u64>,
}

impl<'elf> Parser<'elf> {
    /// Creates a new Parser from elf data.
    pub fn new<R: ReadRef<'elf>>(data: R) -> Result<Self> {
        let file = object::File::parse(data)?;
        let dwarf = Dwarf::new(&file)?;
        let build_id = file
            .build_id()?
            .ok_or(Error::Custom("Unable to find build ID in elf!"))?;

        let address_size = file.architecture().address_size().ok_or(Error::Custom(
            "Unsupported architecture, no address size information!",
        ))?;

        let metadata_addresses = file
            .symbols()
            .filter(|s| s.name().is_ok_and(|n| n.contains("cdefmt_log_metadata")))
            .map(|s| s.address())
            .collect::<Vec<_>>();

        Ok(Parser {
            logs_section: file
                .section_by_name(".cdefmt")
                .ok_or(Error::MissingSection)?
                .data()?,
            build_id,
            dwarf,
            address_size,
            metadata_addresses,
        })
    }

    /// Returns a specific log's metadata.
    pub fn get_log_metadata(&self, id: usize) -> Result<Metadata<'elf>> {
        parse_metadata(self.logs_section, id, self.endian())
    }

    /// Returns an iterator over all of the log's metadata/type information.
    pub fn iter_logs<'parser>(&'parser self) -> LogIterator<'parser, 'elf> {
        LogIterator {
            parser: self,
            symbol_addr_iterator: self.metadata_addresses.iter(),
        }
    }

    /// Returns the type of the log's arguments.
    /// Return:
    /// * Ok(Some(_)) => The type of the arguments.
    /// * Ok(None)    => Unable to find the type in the elf's dwarf section.
    /// * Err(_)      => Encountered some error while parsing the dwarf.
    pub fn get_log_args_type(&self, metadata: &Metadata) -> Result<Option<Type>> {
        let type_name = format!("cdefmt_log_args_t{}", metadata.counter);
        self.dwarf.get_type(metadata.file, &type_name)
    }

    pub fn build_id(&self) -> &'elf [u8] {
        self.build_id
    }

    pub fn address_size(&self) -> AddressSize {
        self.address_size
    }

    pub fn endian(&self) -> gimli::RunTimeEndian {
        self.dwarf.endian()
    }
}

pub struct LogIterator<'parser, 'elf>
where
    'elf: 'parser,
{
    parser: &'parser Parser<'elf>,
    symbol_addr_iterator: std::slice::Iter<'parser, u64>,
}

impl<'elf> Iterator for LogIterator<'_, 'elf> {
    type Item = Result<(Metadata<'elf>, Option<Type>)>;

    fn next(&mut self) -> Option<Self::Item> {
        let metadata = match self.symbol_addr_iterator.next() {
            Some(&addr) => match self.parser.get_log_metadata(addr as usize) {
                Ok(m) => m,
                Err(e) => return Some(Err(e)),
            },
            None => return None,
        };

        let ty = match self.parser.get_log_args_type(&metadata) {
            Ok(t) => t,
            Err(e) => return Some(Err(e)),
        };

        Some(Ok((metadata, ty)))
    }
}
