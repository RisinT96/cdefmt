use std::collections::BTreeMap;

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
    Structure(Vec<StructureMember>),
    Pointer(Box<Type>),
}

#[derive(Debug, Clone)]
pub struct StructureMember {
    pub offset: u64,
    pub name: String,
    pub ty: Type,
}
