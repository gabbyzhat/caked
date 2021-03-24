/// Floating point data.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Floating {
    /// A 32-bit floating point.
    F32(f32),

    /// A 64-bit floating point.
    F64(f64),
}

impl Eq for Floating {}
impl Ord for Floating {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

/// Integral data.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Integral {
    /// A u8.
    U8(u8),

    /// An i8.
    I8(i8),

    /// A u16.
    U16(u16),

    /// An i16.
    I16(i16),

    /// A u32.
    U32(u32),

    /// An i32.
    I32(i32),

    /// A u64.
    U64(u64),

    /// An i64.
    I64(i64),

    /// A usize.
    US(usize),

    /// An isize.
    IS(isize),
}
