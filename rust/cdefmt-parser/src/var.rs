use std::collections::BTreeMap;

use gimli::Reader;

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
    pub fn parse<R: Reader>(ty: &Type, data: &mut R) -> Result<Self> {
        Ok(match ty {
            Type::Bool => Var::Bool(data.read_u8()? == 0),
            Type::U8 => Var::U8(data.read_u8()?),
            Type::U16 => Var::U16(data.read_u16()?),
            Type::U32 => Var::U32(data.read_u32()?),
            Type::U64 => Var::U64(data.read_u64()?),
            Type::I8 => Var::I8(data.read_i8()?),
            Type::I16 => Var::I16(data.read_i16()?),
            Type::I32 => Var::I32(data.read_i32()?),
            Type::I64 => Var::I64(data.read_i64()?),
            Type::F32 => Var::F32(data.read_f32()?),
            Type::F64 => Var::F64(data.read_f64()?),
            Type::Enumeration {
                ty: inner_type,
                valid_values,
            } => Var::Enumeration {
                ty: ty.clone(),
                value: Box::new(Self::parse(&inner_type, data)?),
            },
            Type::Structure { name, members } => Var::Structure {
                members: members
                    .iter()
                    .map(|m| Self::parse(&m.ty, data))
                    .collect::<Result<Vec<_>>>()?,
            },
            Type::Pointer(ty) => Var::Pointer(Box::new(Self::parse(ty, data)?)),
        })
    }
}
