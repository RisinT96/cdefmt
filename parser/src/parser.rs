//! Contains logic related to finding logs in the elf and parsing them.

use std::collections::HashMap;

use gimli::Reader;
use object::{AddressSize, Object, ObjectSection, ReadRef};

use crate::{
    dwarf::Dwarf,
    log::{Log, MetadataV1, Schema},
    r#type::Type,
    var::Var,
    Error, Result,
};

/// Responsible for parsing logs from the elf.
pub struct Parser<'data> {
    log_cache: HashMap<usize, (MetadataV1, Type)>,
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
        let build_id = file.build_id()?.unwrap();

        let address_size = file.architecture().address_size().unwrap();
        Ok(Parser {
            log_cache: Default::default(),
            logs_section: file
                .section_by_name(".cdefmt")
                .ok_or(Error::MissingSection)?
                .data()?,
            build_id,
            dwarf,
            address_size,
        })
    }

    // Parses a log and returns it.
    pub fn parse_log(&mut self, data: &[u8]) -> Result<Log> {
        let mut data = gimli::EndianSlice::new(data, self.dwarf.endian());
        // TODO: Make safer, maybe switch to u64 everywhere.
        let log_id = data.read_address(self.address_size.bytes())? as usize;

        if log_id >= self.logs_section.len() {
            return Err(Error::OutOfBounds(log_id, self.logs_section.len()));
        }

        if !self.log_cache.contains_key(&log_id) {
            let log_info = self.parse_log_metadata(log_id)?;
            let ty = self.parse_log_args_type(&log_info)?;
            self.log_cache.insert(log_id, (log_info, ty));
        };

        let (metadata, ty) = self.log_cache.get(&log_id).unwrap();

        let args = self.parse_log_args(ty, data)?;
        let log = Log::new(metadata.clone(), args);

        if log_id == 0 {
            self.validate_init(&log)?
        }

        Ok(log)
    }

    // Parses the log's static information.
    fn parse_log_metadata(&self, log_id: usize) -> Result<MetadataV1> {
        let json = &self.logs_section[log_id..]
            .split(|b| *b == 0)
            .next()
            .ok_or(Error::NoNullTerm)?;
        let json = std::str::from_utf8(json).map_err(|e| Error::Utf8(log_id, e))?;
        let schema: Schema = serde_json::from_str(json)?;

        let mut metadata = match schema.schema {
            1 => serde_json::from_str::<MetadataV1>(json),
            _ => return Err(Error::Schema(schema.schema)),
        }?;

        metadata.id = log_id;

        Ok(metadata)
    }

    // Parses the log's arguments.
    fn parse_log_args_type(&self, metadata: &MetadataV1) -> Result<Type> {
        let type_name = format!("cdefmt_log_args_t{}", metadata.counter);
        self.dwarf
            .get_type(&metadata.file, &type_name)
            .transpose()
            .unwrap()
    }

    // Parses the log's arguments.
    fn parse_log_args<R: Reader>(&self, ty: &Type, mut data: R) -> Result<Vec<Var>> {
        let members = if let Type::Structure(members) = ty {
            members
        } else {
            return Err(Error::Custom("The log's args aren't a structure!"));
        };

        // Parse the raw data into `Var` representation.
        members
            .iter()
            // Due to the way the log is constructed in the c code, the first argument is always the
            // log id, we already have it.
            .skip(1)
            .map(|m| Ok(Var::parse(&m.ty, &mut data)?.0))
            .collect()
    }

    fn validate_init(&self, log: &Log) -> Result<()> {
        let args = log.get_args();
        if let Some(Var::Array(build_id)) = args.first() {
            let build_id = build_id
                .iter()
                .map(|b| match b {
                    Var::U8(b) => Ok(*b),
                    _ => Err(Error::Custom("Build ID data contains non u8 element!")),
                })
                .collect::<Result<Vec<_>>>()?;
            if self.build_id != build_id {
                Err(Error::Custom("Build ID mismatch!"))
            } else {
                Ok(())
            }
        } else {
            Err(Error::Custom("Build ID missing or not an array"))
        }
    }
}
