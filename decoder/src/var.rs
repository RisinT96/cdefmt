use std::collections::BTreeMap;

use cdefmt_parser::r#type::Type;
use gimli::{Reader, ReaderOffset};

use crate::{
    format::{DisplayHint, DisplayType},
    Result,
};

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

macro_rules! format_common {
    // Width and precision
    ($align:literal, $sign:literal, $alternate:literal, $zero_pad:literal, $width:ident, $precision:ident, $ty:literal, $val:ident) => {
        ::std::format!(
            ::std::concat!("{:", $align, $sign, $alternate, $zero_pad, "w$", ".p$", $ty, "}"),
            $val,
            w = $width,
            p = $precision
        )
    };
    // Only width
    ($align:literal, $sign:literal, $alternate:literal, $zero_pad:literal, "w", $width:ident, $ty: literal, $val:ident) => {
        ::std::format!(
            ::std::concat!("{:", $align, $sign, $alternate, $zero_pad, "w$", $ty, "}"),
            $val,
            w = $width
        )
    };
    // Only precision
    ($align:literal, $sign:literal, $alternate:literal, $zero_pad:literal, "p", $precision:ident, $ty: literal, $val:ident) => {
        ::std::format!(
            ::std::concat!("{:", $align, $sign, $alternate, $zero_pad, ".p$", $ty, "}"),
            $val,
            p = $precision
        )
    };
    // No width nor precision
    ($align:literal, $sign:literal, $alternate:literal, $zero_pad:literal, $ty: literal, $val:ident) => {
        ::std::format!(
            ::std::concat!("{:", $align, $sign, $alternate, $zero_pad, $ty, "}"),
            $val
        )
    };
}

macro_rules! format_match {
    // Match on width and precision
    ($align:literal, $sign:literal, $alternate:literal, $zero_pad:literal, $width:ident, $precision:ident, $ty:literal, $val:ident) => {
        match ($width, $precision) {
            (None, None) => format_common!($align, $sign, $alternate, $zero_pad, $ty, $val),
            (None, Some(p)) => {
                format_common!($align, $sign, $alternate, $zero_pad, "p", p, $ty, $val)
            }
            (Some(w), None) => {
                format_common!($align, $sign, $alternate, $zero_pad, "w", w, $ty, $val)
            }
            (Some(w), Some(p)) => {
                format_common!($align, $sign, $alternate, $zero_pad, w, p, $ty, $val)
            }
        }
    };
    // Match on zero pad
    ($align:literal, $sign:literal, $alternate:literal, $zero_pad:ident, $width:ident, $precision:ident, $ty:literal, $val:ident) => {
        match ($zero_pad) {
            true => format_match!($align, $sign, $alternate, "0", $width, $precision, $ty, $val),
            false => format_match!($align, $sign, $alternate, "", $width, $precision, $ty, $val),
        }
    };
    // Match on alternate
    ($align:literal, $sign:literal, $alternate:ident, $zero_pad:ident, $width:ident, $precision:ident, $ty:literal, $val:ident) => {
        match ($alternate) {
            true => format_match!($align, $sign, "#", $zero_pad, $width, $precision, $ty, $val),
            false => format_match!($align, $sign, "", $zero_pad, $width, $precision, $ty, $val),
        }
    };
    // Match on sign
    ($align:literal, $sign:ident, $alternate:ident, $zero_pad:ident, $width:ident, $precision:ident, $ty:literal, $val:ident) => {
        match ($sign) {
            true => {
                format_match!($align, "+", $alternate, $zero_pad, $width, $precision, $ty, $val)
            }
            false => {
                format_match!($align, "", $alternate, $zero_pad, $width, $precision, $ty, $val)
            }
        }
    };
    // Match on alignment
    ($align:ident, $sign:ident, $alternate:ident, $zero_pad:ident, $width:ident, $precision:ident, $ty:literal, $val:ident) => {
        match ($align) {
            None => format_match!("", $sign, $alternate, $zero_pad, $width, $precision, $ty, $val),
            Some(std::fmt::Alignment::Left) => {
                format_match!("<", $sign, $alternate, $zero_pad, $width, $precision, $ty, $val)
            }
            Some(std::fmt::Alignment::Center) => {
                format_match!("^", $sign, $alternate, $zero_pad, $width, $precision, $ty, $val)
            }
            Some(std::fmt::Alignment::Right) => {
                format_match!(">", $sign, $alternate, $zero_pad, $width, $precision, $ty, $val)
            }
        }
    };
    // No precision
    ($align:ident, $sign:ident, $alternate:ident, $zero_pad:ident, $width:ident, $ty:literal, $val:ident) => {
        format_match!($align, $sign, $alternate, $zero_pad, $width, None, $ty, $val)
    };
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

    pub fn format(&self, hint: &DisplayHint) -> String {
        match (self, &hint.ty) {
            (Var::U8(val), &DisplayType::String) => {
                String::from_utf8_lossy(&[*val]).to_string()
            }
            (Var::I8(val), &DisplayType::String) => {
                String::from_utf8_lossy(&[*val as u8]).to_string()
            }
            (Var::Bool(true), _) => "true".to_string(),
            (Var::Bool(false), _) => "false".to_string(),
            (Var::U8(val), _) => Self::format_inner(val, hint),
            (Var::U16(val), _) => Self::format_inner(val, hint),
            (Var::U32(val), _) => Self::format_inner(val, hint),
            (Var::U64(val), _) => Self::format_inner(val, hint),
            (Var::I8(val), _) => Self::format_inner(val, hint),
            (Var::I16(val), _) => Self::format_inner(val, hint),
            (Var::I32(val), _) => Self::format_inner(val, hint),
            (Var::I64(val), _) => Self::format_inner(val, hint),
            (Var::F32(val), _) => Self::format_float(*val, hint),
            (Var::F64(val), _) => Self::format_float(*val, hint),
            (
                Var::Enumeration {
                    value,
                    valid_values,
                },
                _,
            ) => {
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
            (Var::Structure { members }, _) => {
                let (start, join, end) = if hint.alternate {
                    ("{\n\t", ",\n\t", "\n}")
                } else {
                    ("{", ", ", "}")
                };
                start.to_string()
                    + &members
                        .iter()
                        .map(|m| format!("{}: {}", m.name, m.value.format(hint)))
                        .collect::<Vec<_>>()
                        .join(join)
                    + end
            }
            (Var::Pointer(value), _) => {
                // Override hints for pointer types.
                let hint = DisplayHint {
                    align: None,
                    sign: false,
                    alternate: true,
                    zero_pad: false,
                    width: None,
                    precision: None,
                    ty: DisplayType::Pointer,
                };
                value.format(&hint)
            }
            (Var::Array(values), _) => {
                let (start, join, end) = match hint.ty {
                    DisplayType::String => ("", "", ""),
                    _ => ("[", ", ", "]"),
                };

                start.to_string()
                    + &values
                        .iter()
                        .map(|v| v.format(hint))
                        .collect::<Vec<_>>()
                        .join(join)
                    + end
            }
        }
    }

    fn format_inner<T>(val: T, hint: &DisplayHint) -> String
    where
        T: std::fmt::Binary
            + std::fmt::Debug
            + std::fmt::Display
            + std::fmt::LowerExp
            + std::fmt::LowerHex
            + std::fmt::Octal
            + std::fmt::UpperExp
            + std::fmt::UpperHex,
    {
        let DisplayHint {
            align,
            sign,
            alternate,
            zero_pad,
            width,
            precision,
            ty,
        } = hint;

        match ty {
            DisplayType::Binary => format_match!(align, sign, alternate, zero_pad, width, "b", val),
            DisplayType::Debug | DisplayType::Display => {
                format_match!(align, sign, alternate, zero_pad, width, precision, "", val)
            }
            DisplayType::LowerExp => {
                format_match!(align, sign, alternate, zero_pad, width, precision, "e", val)
            }
            DisplayType::LowerHex => {
                format_match!(align, sign, alternate, zero_pad, width, "x", val)
            }
            DisplayType::Octal => format_match!(align, sign, alternate, zero_pad, width, "o", val),
            DisplayType::Pointer => {
                let width = Some(std::mem::size_of::<T>());
                format_match!("", "", "#", "0", width, None, "x", val)
            }
            DisplayType::UpperExp => {
                format_match!(align, sign, alternate, zero_pad, width, precision, "E", val)
            }
            DisplayType::UpperHex => {
                format_match!(align, sign, alternate, zero_pad, width, "X", val)
            }
            _ => unreachable!("Unexpected hint: {:?}", ty),
        }
    }

    fn format_float<F>(val: F, hint: &DisplayHint) -> String
    where
        F: std::fmt::Debug + std::fmt::Display + std::fmt::LowerExp + std::fmt::UpperExp,
    {
        let DisplayHint {
            align,
            sign,
            alternate,
            zero_pad,
            width,
            precision,
            ty,
        } = hint;

        match ty {
            DisplayType::Debug | DisplayType::Display => {
                format_match!(align, sign, alternate, zero_pad, width, precision, "", val)
            }
            DisplayType::LowerExp => {
                format_match!(align, sign, alternate, zero_pad, width, precision, "e", val)
            }
            DisplayType::UpperExp => {
                format_match!(align, sign, alternate, zero_pad, width, precision, "E", val)
            }
            DisplayType::Binary => format!("Unable to format [{val}] as Binary!"),
            DisplayType::LowerHex => format!("Unable to format [{val}] as LowerHex!"),
            DisplayType::Octal => format!("Unable to format [{val}] as Octal!"),
            DisplayType::String => format!("Unable to format [{val}] as String!"),
            DisplayType::Pointer => format!("Unable to format [{val}] as Pointer!"),
            DisplayType::UpperHex => format!("Unable to format [{val}] as UpperHex!"),
        }
    }

    pub fn as_u64(&self) -> u64 {
        match self {
            Var::Bool(v) => *v as u64,
            Var::U8(v) => *v as u64,
            Var::U16(v) => *v as u64,
            Var::U32(v) => *v as u64,
            Var::U64(v) => *v ,
            Var::I8(v) => *v as u64,
            Var::I16(v) => *v as u64,
            Var::I32(v) => *v as u64,
            Var::I64(v) => *v as u64,
            Var::F32(v) => *v as u64,
            Var::F64(v) => *v as u64,
            _ => todo!("Should probably return an Option here or something"),
        }
    }
}
