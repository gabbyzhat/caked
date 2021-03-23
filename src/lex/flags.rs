use std::default::Default;

/// Flags for lexer.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Flags {
    /// Use signed when possible.
    pub signed: bool,

    /// Use f32 over f64 when possible.
    pub use_f32: bool,

    /// Use i/u8 over size when possible.
    pub use_8: bool,

    /// Use i/u16 over size when possible.
    pub use_16: bool,

    /// Use i/u32 over size when possible.
    pub use_32: bool,

    /// Use i/u64 over size when possible.
    pub use_64: bool,
}

impl Default for Flags {
    fn default() -> Self {
        Self {signed: true, use_f32: false, use_8: false, use_16: false, use_32: true, use_64: false}
    }
}
