
use std::default::Default;

/// Flags for the lexer.
#[derive(Debug)]
pub struct Flags {
    /// Prefer signed over unsigned.
    pub signed: bool,

    /// Attempt an f32 before an f64.
    pub float: bool,
    /// Attempt a (ui)8 before a (ui)size.
    pub byte: bool,

    /// Attempt a (ui)16 before a (ui)size.
    pub short: bool,

    /// Attempt a (ui)32 before a (ui)size.
    pub long: bool,

    /// Attempt a (ui)64 before a (ui)size.
    pub llong: bool,
}

impl Default for Flags {
    fn default() -> Self {
        Self {
            signed: false,
            float: true,
            byte: false,
            short: false,
            long: true,
            llong: true,
        }
    }
}

impl Flags {
    /// Creates a new set of flags.
    pub fn new(
        signed: bool,
        float: bool,
        byte: bool,
        short: bool,
        long: bool,
        llong: bool,
    ) -> Self {
        Self {
            signed,
            float,
            byte,
            short,
            long,
            llong,
        }
    }
}

/// A lexer token.
#[derive(Debug)]
pub enum Token {
    /// A separator. (,)
    Sep,
    
    /// Assignment of value to key. (k=>v)
    Assign,

    /// Set opening. ([)
    Open,

    /// Set closing. (])
    Close,

    /// Integral value.
    Int(String),

    /// Floating point value.
    Float(String),

    /// Identifier.
    Id(String),

    /// Single quoted literal.
    SQL(String),

    /// Double quoted literal.
    DQL(String),

    /// EOT literal.
    EOT(String),
}