use std::default::Default;

use enumflags2::{bitflags, make_bitflags};



/// Flags for lexer.
#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Flags {
    /// Use signed when possible.
    Signed = 1 << 0,

    /// Use f32 over f64 when possible.
    F32 = 1 << 2,

    /// Use i/u8 over size when possible.
    I8 = 1 << 3,

    /// Use i/u16 over size when possible.
    I16 = 1 << 4,

    /// Use i/u32 over size when possible.
    I32 = 1 << 5,

    /// Use i/u64 over size when possible.
    I64 = 1 << 6,
}

impl Default for Flags {
    fn default() -> Self {
        make_bitflags!(Flags::{Signed | I32})
    }
}
