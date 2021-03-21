/// A lexer token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// A separator. (,)
    Separator,
    /// Assignment of value to key. (k=>v)
    Assignment,

    /// Set opening. ([)
    OpenSet,

    /// Set closing. (])
    CloseSet,

    /// Integral value.
    Integral(String),

    /// Floating point value.
    Float(String),

    /// Identifier.
    Identifier(String),

    /// Single quoted literal.
    SingleQuote(String),

    /// Double quoted literal.
    DoubleQuote(String),

    /// Heredoc literal.
    Heredoc(String),
}
