use std::collections::BTreeMap;

// TODO: support booleans larger than 1 byte?

#[derive(Debug, Clone)]
pub enum Type {
    Bool,
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Enumeration {
        ty: Box<Type>,
        /// Use i128 to deal with u64 and i64 enums.
        valid_values: BTreeMap<i128, String>,
    },
    Structure {
        members: Vec<StructureMember>,
        size: usize,
    },
    Pointer(Box<Type>),
    Array {
        ty: Box<Type>,
        lengths: Vec<u64>,
    },
}

#[derive(Debug, Clone)]
pub struct StructureMember {
    pub offset: u64,
    pub name: String,
    pub ty: Type,
}

impl Type {
    pub fn size(&self) -> usize {
        match self {
            Type::Bool => 1,
            Type::U8 => 1,
            Type::U16 => 2,
            Type::U32 => 4,
            Type::U64 => 8,
            Type::I8 => 1,
            Type::I16 => 2,
            Type::I32 => 4,
            Type::I64 => 8,
            Type::F32 => 4,
            Type::F64 => 8,
            Type::Enumeration { ty, .. } => ty.size(),
            Type::Structure { size, .. } => {
                return *size;
            }
            Type::Pointer(ty) => ty.size(),
            Type::Array { ty, lengths } => {
                if lengths.len() == 0 {
                    return 0;
                }

                ty.size() * (lengths.iter().fold(1, |acc, x| acc * x) as usize)
            }
        }
    }
}
