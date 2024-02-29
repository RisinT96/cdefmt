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
    Structure{name: String, members: Vec<StructureMember>},
    Pointer(u64),
}

#[derive(Debug, Clone)]
pub struct StructureMember {
    pub name: String,
    pub ty: Type,
}