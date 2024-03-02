use std::collections::BTreeMap;

use gimli::{Reader, ReaderOffset};

use crate::{r#type::Type, Error, Result};

#[derive(Debug, Clone)]
pub enum Var {
    Bool(bool),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Enumeration { ty: Type, value: Box<Var> },
    Structure { members: Vec<Var> },
    Pointer(Box<Var>),
}

impl Var {
    pub fn parse<R: Reader>(ty: &Type, data: &mut R) -> Result<(Self, u64)> {
        Ok(match ty {
            Type::Bool => (Var::Bool(data.read_u8()? == 0), 1),
            Type::U8 => (Var::U8(data.read_u8()?), 1),
            Type::U16 => (Var::U16(data.read_u16()?), 2),
            Type::U32 => (Var::U32(data.read_u32()?), 4),
            Type::U64 => (Var::U64(data.read_u64()?), 8),
            Type::I8 => (Var::I8(data.read_i8()?), 1),
            Type::I16 => (Var::I16(data.read_i16()?), 2),
            Type::I32 => (Var::I32(data.read_i32()?), 4),
            Type::I64 => (Var::I64(data.read_i64()?), 8),
            Type::F32 => (Var::F32(data.read_f32()?), 4),
            Type::F64 => (Var::F64(data.read_f64()?), 8),
            Type::Enumeration {
                ty: inner_type,
                valid_values,
            } => {
                let (value, bytes) = Self::parse(&inner_type, data)?;
                (
                    Var::Enumeration {
                        ty: ty.clone(),
                        value: Box::new(value),
                    },
                    bytes,
                )
            }
            Type::Structure { name, members } => {
                let mut offset = 0;
                let members = members
                    .iter()
                    .map(|m| -> Result<Self> {
                        if m.offset > offset {
                            data.skip(ReaderOffset::from_u64(m.offset - offset)?)?;
                        }

                        let (var, bytes) = Self::parse(&m.ty, data)?;
                        offset += bytes;

                        Ok(var)
                    })
                    .collect::<Result<Vec<_>>>()?;
                (Var::Structure { members }, offset)
            }
            Type::Pointer(ty) => {
                let (value, bytes) = Self::parse(ty, data)?;
                (Var::Pointer(Box::new(value)), bytes)
            }
        })
    }
}
