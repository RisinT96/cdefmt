use std::collections::BTreeMap;

use gimli::{Reader, ReaderOffset};

use crate::{r#type::Type, Result};

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
    Enumeration {
        value: Box<Var>,
        valid_values: BTreeMap<i128, String>,
    },
    Structure {
        members: Vec<Var>,
    },
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
                let (value, bytes) = Self::parse(inner_type, data)?;
                (
                    Var::Enumeration {
                        value: Box::new(value),
                        valid_values: valid_values.clone(),
                    },
                    bytes,
                )
            }
            Type::Structure(members) => {
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

    pub fn format(&self) -> String {
        match self {
            Var::Bool(true) => "true".to_string(),
            Var::Bool(false) => "false".to_string(),
            Var::U8(val) => format!("{val}"),
            Var::U16(val) => format!("{val}"),
            Var::U32(val) => format!("{val}"),
            Var::U64(val) => format!("{val}"),
            Var::I8(val) => format!("{val}"),
            Var::I16(val) => format!("{val}"),
            Var::I32(val) => format!("{val}"),
            Var::I64(val) => format!("{val}"),
            Var::F32(val) => format!("{val}"),
            Var::F64(val) => format!("{val}"),
            Var::Enumeration {
                value,
                valid_values,
            } => {
                let value = match value.as_ref() {
                    Var::U8(v) => *v as i128,
                    Var::U16(v) => *v as i128,
                    Var::U32(v) => *v as i128,
                    Var::U64(v) => *v as i128,
                    Var::I8(v) => *v as i128,
                    Var::I16(v) => *v as i128,
                    Var::I32(v) => *v as i128,
                    Var::I64(v) => *v as i128,
                    _ => unreachable!("C enums must have integer types!"),
                };

                valid_values
                    .get(&value)
                    .map(|name| name.to_owned())
                    .unwrap_or_else(|| value.to_string())
            }
            Var::Structure { members } => {
                "[".to_string()
                    + &members
                        .iter()
                        .map(|m| m.format())
                        .collect::<Vec<_>>()
                        .join(", ")
                    + "]"
            }
            Var::Pointer(value) => value.format(),
        }
    }
}
