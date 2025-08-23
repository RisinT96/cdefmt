use std::collections::BTreeMap;

use cdefmt_parser::r#type::Type;
use gimli::{Reader, ReaderOffset};

use crate::Result;

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
        members: Vec<StructureMember>,
    },
    Pointer(Box<Var>),
    Array(Vec<Var>),
}

#[derive(Debug, Clone)]
pub struct StructureMember {
    pub name: String,
    pub value: Var,
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
            Type::Structure { members, size } => {
                let mut total_offset = 0;
                let members = members
                    .iter()
                    .map(|m| -> Result<StructureMember> {
                        if m.offset > total_offset {
                            let bytes_to_skip = m.offset - total_offset;
                            data.skip(ReaderOffset::from_u64(bytes_to_skip)?)?;
                            total_offset += bytes_to_skip;
                        }

                        let (var, bytes) = Self::parse(&m.ty, data)?;
                        total_offset += bytes;

                        Ok(StructureMember {
                            name: m.name.clone(),
                            value: var,
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;

                let bytes_to_skip = *size as u64 - total_offset;
                data.skip(ReaderOffset::from_u64(bytes_to_skip)?)?;

                (Var::Structure { members }, *size as u64)
            }
            Type::Pointer(ty) => {
                let (value, bytes) = Self::parse(ty, data)?;
                (Var::Pointer(Box::new(value)), bytes)
            }
            Type::Array { ty, lengths } => {
                let l = lengths[0];

                let mut values = Vec::with_capacity(l as usize);
                for _ in 0..l {
                    let (val, _) = Self::parse(ty, data)?;
                    values.push(val);
                }

                (Var::Array(values), 0)
            }
        })
    }

    pub fn as_u64(&self) -> u64 {
        match self {
            Var::Bool(v) => *v as u64,
            Var::U8(v) => *v as u64,
            Var::U16(v) => *v as u64,
            Var::U32(v) => *v as u64,
            Var::U64(v) => *v,
            Var::I8(v) => *v as u64,
            Var::I16(v) => *v as u64,
            Var::I32(v) => *v as u64,
            Var::I64(v) => *v as u64,
            Var::F32(v) => *v as u64,
            Var::F64(v) => *v as u64,
            _ => todo!("Should probably return an Option here or something"),
        }
    }

    pub fn as_i128(&self) -> i128 {
        match self {
            Var::Bool(v) => *v as i128,
            Var::U8(v) => *v as i128,
            Var::U16(v) => *v as i128,
            Var::U32(v) => *v as i128,
            Var::U64(v) => *v as i128,
            Var::I8(v) => *v as i128,
            Var::I16(v) => *v as i128,
            Var::I32(v) => *v as i128,
            Var::I64(v) => *v as i128,
            Var::F32(v) => *v as i128,
            Var::F64(v) => *v as i128,
            _ => todo!("Should probably return an Option here or something"),
        }
    }

    fn format_as_string(&self) -> rformat::error::Result<String> {
        match self {
            Var::U8(v) => Ok(String::from_utf8_lossy(&[*v]).to_string()),
            Var::I8(v) => Ok(String::from_utf8_lossy(&[*v as u8]).to_string()),
            Var::Array(elements) => Ok(elements
                .iter()
                .map(|e| e.format_as_string())
                .collect::<rformat::error::Result<Vec<_>>>()?
                .join("")),
            _ => Err(rformat::error::FormatError::Custom(format!(
                "Can't format {:?} as string!",
                self
            ))),
        }
    }
}

macro_rules! format_enumeration {
    ($f: expr, $value: expr, $valid_values: expr) => {{
        let value = $value.as_i128();
        if let Some(name) = $valid_values.get(&value) {
            write!($f, "{}(", name)?;
            value.fmt($f)?;
            write!($f, ")")
        } else {
            write!($f, "Unknown(")?;
            value.fmt($f)?;
            write!($f, ")")
        }
    }};
}

macro_rules! format_structure {
    ($f: expr, $members: expr) => {{
        write!($f, "{{ ")?;
        for (i, member) in $members.iter().enumerate() {
            if i != 0 {
                write!($f, ", ")?;
            }

            write!($f, "{}: ", member.name)?;
            member.value.fmt($f)?;
        }
        write!($f, " }}")
    }};
}

macro_rules! format_array {
    ($f: expr, $elements: expr) => {{
        write!($f, "[")?;
        for (i, elem) in $elements.iter().enumerate() {
            if i != 0 {
                write!($f, ", ")?;
            }
            elem.fmt($f)?;
        }
        write!($f, "]")
    }};
}

impl core::fmt::Binary for Var {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Var::U8(v) => v.fmt(f),
            Var::U16(v) => v.fmt(f),
            Var::U32(v) => v.fmt(f),
            Var::U64(v) => v.fmt(f),
            Var::I8(v) => v.fmt(f),
            Var::I16(v) => v.fmt(f),
            Var::I32(v) => v.fmt(f),
            Var::I64(v) => v.fmt(f),
            Var::Enumeration {
                value,
                valid_values,
            } => format_enumeration!(f, value, valid_values),
            Var::Structure { members } => format_structure!(f, members),
            Var::Pointer(inner) => inner.fmt(f),
            Var::Array(elements) => format_array!(f, elements),
            _ => write!(f, "Can't format {:?} as binary!", self),
        }
    }
}

/* Debug is implemented using derive */

impl core::fmt::Display for Var {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Var::Bool(v) => v.fmt(f),
            Var::U8(v) => v.fmt(f),
            Var::U16(v) => v.fmt(f),
            Var::U32(v) => v.fmt(f),
            Var::U64(v) => v.fmt(f),
            Var::I8(v) => v.fmt(f),
            Var::I16(v) => v.fmt(f),
            Var::I32(v) => v.fmt(f),
            Var::I64(v) => v.fmt(f),
            Var::F32(v) => v.fmt(f),
            Var::F64(v) => v.fmt(f),
            Var::Enumeration {
                value,
                valid_values,
            } => format_enumeration!(f, value, valid_values),
            Var::Structure { members } => format_structure!(f, members),
            Var::Pointer(inner) => inner.fmt(f),
            Var::Array(elements) => format_array!(f, elements),
        }
    }
}

impl core::fmt::LowerExp for Var {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Var::U8(v) => v.fmt(f),
            Var::U16(v) => v.fmt(f),
            Var::U32(v) => v.fmt(f),
            Var::U64(v) => v.fmt(f),
            Var::I8(v) => v.fmt(f),
            Var::I16(v) => v.fmt(f),
            Var::I32(v) => v.fmt(f),
            Var::I64(v) => v.fmt(f),
            Var::F32(v) => v.fmt(f),
            Var::F64(v) => v.fmt(f),
            Var::Structure { members } => format_structure!(f, members),
            Var::Array(elements) => format_array!(f, elements),
            _ => write!(f, "Can't format {:?} as lower exponential!", self),
        }
    }
}

impl core::fmt::LowerHex for Var {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Var::U8(v) => v.fmt(f),
            Var::U16(v) => v.fmt(f),
            Var::U32(v) => v.fmt(f),
            Var::U64(v) => v.fmt(f),
            Var::I8(v) => v.fmt(f),
            Var::I16(v) => v.fmt(f),
            Var::I32(v) => v.fmt(f),
            Var::I64(v) => v.fmt(f),
            Var::Enumeration {
                value,
                valid_values,
            } => format_enumeration!(f, value, valid_values),
            Var::Structure { members } => format_structure!(f, members),
            Var::Pointer(inner) => inner.fmt(f),
            Var::Array(elements) => format_array!(f, elements),
            _ => write!(f, "Can't format {:?} as lower hexadecimal!", self),
        }
    }
}

impl core::fmt::Octal for Var {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Var::U8(v) => v.fmt(f),
            Var::U16(v) => v.fmt(f),
            Var::U32(v) => v.fmt(f),
            Var::U64(v) => v.fmt(f),
            Var::I8(v) => v.fmt(f),
            Var::I16(v) => v.fmt(f),
            Var::I32(v) => v.fmt(f),
            Var::I64(v) => v.fmt(f),
            Var::Enumeration {
                value,
                valid_values,
            } => format_enumeration!(f, value, valid_values),
            Var::Structure { members } => format_structure!(f, members),
            Var::Pointer(inner) => inner.fmt(f),
            Var::Array(elements) => format_array!(f, elements),
            _ => write!(f, "Can't format {:?} as octal!", self),
        }
    }
}

impl core::fmt::Pointer for Var {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Var::U8(v) => ((*v) as *const u8).fmt(f),
            Var::U16(v) => ((*v) as *const u16).fmt(f),
            Var::U32(v) => ((*v) as *const u32).fmt(f),
            Var::U64(v) => ((*v) as *const u64).fmt(f),
            Var::I8(v) => ((*v) as *const i8).fmt(f),
            Var::I16(v) => ((*v) as *const i16).fmt(f),
            Var::I32(v) => ((*v) as *const i32).fmt(f),
            Var::I64(v) => ((*v) as *const i64).fmt(f),
            Var::Structure { members } => format_structure!(f, members),
            Var::Pointer(inner) => (*inner).fmt(f),
            Var::Array(elements) => format_array!(f, elements),
            _ => write!(f, "Can't format {:?} as pointer!", self),
        }
    }
}

impl core::fmt::UpperExp for Var {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Var::U8(v) => v.fmt(f),
            Var::U16(v) => v.fmt(f),
            Var::U32(v) => v.fmt(f),
            Var::U64(v) => v.fmt(f),
            Var::I8(v) => v.fmt(f),
            Var::I16(v) => v.fmt(f),
            Var::I32(v) => v.fmt(f),
            Var::I64(v) => v.fmt(f),
            Var::F32(v) => v.fmt(f),
            Var::F64(v) => v.fmt(f),
            Var::Enumeration {
                value,
                valid_values,
            } => format_enumeration!(f, value, valid_values),
            Var::Structure { members } => format_structure!(f, members),
            Var::Pointer(inner) => inner.fmt(f),
            Var::Array(elements) => format_array!(f, elements),
            _ => write!(f, "Can't format {:?} as upper exponential!", self),
        }
    }
}

impl core::fmt::UpperHex for Var {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Var::U8(v) => v.fmt(f),
            Var::U16(v) => v.fmt(f),
            Var::U32(v) => v.fmt(f),
            Var::U64(v) => v.fmt(f),
            Var::I8(v) => v.fmt(f),
            Var::I16(v) => v.fmt(f),
            Var::I32(v) => v.fmt(f),
            Var::I64(v) => v.fmt(f),
            Var::Enumeration {
                value,
                valid_values,
            } => format_enumeration!(f, value, valid_values),
            Var::Structure { members } => format_structure!(f, members),
            Var::Pointer(inner) => inner.fmt(f),
            Var::Array(elements) => format_array!(f, elements),
            _ => write!(f, "Can't format {:?} as upper hexadecimal!", self),
        }
    }
}

impl rformat::fmt::Custom for Var {
    fn format(
        &self,
        format_spec: &rformat::format_spec::FormatSpec,
        _precision: Option<usize>,
        _width: usize,
        _parameter: &rformat::fmt::format::Parameter,
    ) -> rformat::error::Result<String> {
        match format_spec.r#type {
            rformat::format_spec::Type::Custom("s") => self.format_as_string(),
            _ => Err(rformat::error::FormatError::UnsupportedFormatType(
                format_spec.r#type.to_str().to_string(),
            )),
        }
    }
}
