//! Contains logic related to finding logs in the elf and parsing them.

use std::collections::HashMap;

use cdefmt_parser::{
    metadata::Metadata,
    r#type::{self, Type},
    Parser,
};
use gimli::Reader;
use object::ReadRef;

use crate::{log::Log, var::Var, Error, Result};

/// Responsible for parsing logs from the elf.
pub struct Decoder<'data> {
    parser: Parser<'data>,
    log_cache: HashMap<usize, (Metadata, Option<Type>)>,
}

impl<'data> Decoder<'data> {
    /// Creates a new Parser from elf data.
    pub fn new<R: ReadRef<'data>>(data: R) -> Result<Self> {
        Ok(Decoder {
            parser: Parser::new(data)?,
            log_cache: Default::default(),
        })
    }

    /// Decodes a raw log
    pub fn decode_log(&mut self, data: &[u8]) -> Result<Log> {
        let mut data = gimli::EndianSlice::new(data, self.parser.endian());
        let id = data.read_address(self.parser.address_size().bytes())? as usize;

        if let std::collections::hash_map::Entry::Vacant(e) = self.log_cache.entry(id) {
            // Parse log metadata and type if we don't have it cached.
            let metadata = self.parser.get_log_metadata(id)?;
            let ty = self.parser.get_log_args_type(&metadata)?;
            e.insert((metadata, ty));
        };

        // Unwrap safety: made sure that the entry exists right above here.
        let (metadata, ty) = self.log_cache.get(&id).unwrap();

        let args = if let Some(ty) = ty {
            Some(Self::decode_log_args(ty, data)?)
        } else {
            None
        };

        let log = Log::new(metadata.clone(), args);

        if id == 0 {
            self.validate_init(&log)?
        }

        Ok(log)
    }

    // Parses the log's arguments.
    fn decode_log_args<R: Reader>(ty: &Type, mut data: R) -> Result<Vec<Var>> {
        let members = if let Type::Structure(members) = ty {
            members
        } else {
            return Err(Error::Custom("The log's args aren't a structure!"));
        };

        // We already read the log_id from the data, skip it.
        let members = &members[1..];

        // Parse the raw data into `Var` representation.
        let mut decoded = members
            .iter()
            // The dynamic data should be at the end, ignore it, we'll come back for it afterwards.
            // Not all logs necessarily have it, so we skip by name, instead of outright ignoring
            // the last element.
            .filter(|m| match m.name.as_str() {
                "dynamic_data" => false,
                _ => true,
            })
            .map(|m| Ok(Var::parse(&m.ty, &mut data)?.0))
            .collect::<Result<Vec<_>>>()?;

        // Decode dynamic members
        for (i, member) in members.iter().enumerate() {
            match member.name.as_str() {
                n if n.contains("dynamic_array") => {
                    decoded[i] = Self::decode_dynamic_array(member, &decoded[i], &mut data)?
                }
                _ => continue,
            }
        }

        Ok(decoded)
    }

    fn validate_init(&self, log: &Log) -> Result<()> {
        let args = log.get_args();
        if args.is_none() {
            return Err(Error::Custom("No build ID argument information!"));
        }

        let args = args.unwrap();

        if let Some(Var::Array(build_id)) = args.first() {
            let build_id = build_id
                .iter()
                .map(|b| match b {
                    Var::U8(b) => Ok(*b),
                    _ => Err(Error::Custom("Build ID data contains non u8 element!")),
                })
                .collect::<Result<Vec<_>>>()?;
            if self.parser.build_id() != build_id {
                Err(Error::Custom("Build ID mismatch!"))
            } else {
                Ok(())
            }
        } else {
            Err(Error::Custom("Build ID missing or not an array"))
        }
    }

    fn decode_dynamic_array<R: Reader>(
        metadata: &r#type::StructureMember,
        value: &Var,
        data: &mut R,
    ) -> Result<Var> {
        // The dynamic_array is structured as:
        // [0] size
        // [1] type

        // Extract size from value that was previously decoded
        let size = match value {
            Var::Structure { members } => &members[0].value,
            _ => return Err(Error::Custom("Dynamic array metadata is not a struct!")),
        }
        .as_u64();

        // Extract type from metadata
        let arr_ty = match &metadata.ty {
            Type::Structure(members) => &members[1].ty,
            _ => return Err(Error::Custom("Dynamic array metadata is not a struct!")),
        };

        let ty = match arr_ty {
            Type::Array { ty, .. } => ty,
            _ => {
                return Err(Error::Custom(
                    "Dynamic array type metadata is not an array!",
                ))
            }
        };

        let dyn_ty = Type::Array {
            ty: ty.clone(),
            lengths: vec![size / ty.size() as u64],
        };

        Ok(Var::parse(&dyn_ty, data)?.0)
    }
}
