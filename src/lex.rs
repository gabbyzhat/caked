use observitor::Observe;
use std::convert::TryFrom;
use std::default::Default;
use std::fmt;
use std::fmt::Display;
use std::result::Result;

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

    /// Integer value.
    Int(String),

    /// Floating point value.
    Float(String),

    /// Identifier.
    Identifier(String),

    /// Single quoted literal.
    SingleQuote(String),

    /// Double quoted literal.
    DoubleQuote(String),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub enum State {
    Initial,
    PrepareComment,
    PrepareAssignment,
    SingleQuote,
    DoubleQuote,
    Integer,
    IntegerDecimal,
    Identifier,
    PrepareOpenTag,
    PrepareCloseTag,
    LineComment,
    MultiLineComment,
    MultiLineCommentPrepareExit,
    Decimal,
    SingleQuoteEscape,
    DoubleQuoteEscape,
    DoubleQuoteEscapeControl,
    DoubleQuoteEscapeHex,
    DoubleQuoteEscapeOctal2,
    DoubleQuoteEscapeHexBound,
    DoubleQuoteEscapeHex2,
    PHPTag0,
    PHPTag1,
    PHPTag2,
    DoubleQuoteEscapeOctal3,
}

impl Default for State {
    fn default() -> Self {
        Self::Initial
    }
}

/// A token position.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub struct Position {
    /// Absolute position of the token.
    pub index: usize,
    /// Line of the token.
    pub line: usize,
    /// Column on the line.
    pub column: usize,
}

impl Default for Position {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Line {}, Column {}", self.line, self.column)
    }
}

impl Position {
    /// Creates a new position.
    pub fn new(index: usize, line: usize, column: usize) -> Self {
        Self {
            index,
            line,
            column,
        }
    }

    /// Advances the position.
    pub fn advance(&mut self, new_line: bool) {
        self.index += 1;
        if new_line {
            self.line += 1;
            self.column = 0;
        } else {
            self.column += 1;
        }
    }
}

/// Kind of lexer error.
#[derive(Debug, Clone, PartialEq, Eq, Copy, PartialOrd, Ord)]
pub enum LexErrorKind {
    /// Premature end-of-file.
    EOF,
    /// Unexpected character.
    Unexpected(char),
    /// Invalid Unicode escape.
    InvalidUnicode(u32),
}

/// Lexer error.
#[derive(Debug, Clone, PartialEq, Eq, Copy, PartialOrd, Ord)]
pub struct LexError {
    /// Position of error.
    pub position: Position,
    state: State,
    /// Kind of error.
    pub kind: LexErrorKind,
}

impl LexError {
    /// Creates a new lexer error.
    pub fn new(position: Position, state: State, kind: LexErrorKind) -> Self {
        Self {
            position,
            state,
            kind,
        }
    }
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let Self {
            position,
            state,
            kind,
        } = self;
        use LexErrorKind as K;
        match kind {
            K::Unexpected('\n') => write!(f, "Unexpected new line"),
            K::Unexpected('\r') => write!(f, "Unexpected carriage return"),
            K::Unexpected('\t') => write!(f, "Unexpected tab"),
            K::Unexpected(' ') => write!(f, "Unexpected space"),
            K::Unexpected(c) => write!(f, "'{}'", c),
            K::EOF => write!(f, "end of file"),
            K::InvalidUnicode(x) => write!(f, "Invalid codepoint U+{:#X}", x),
        }?;
        write!(f, "at line {}, column {} ", position.column, position.line)?;
        write!(f, "in state {:?}", state)
    }
}

/// Feeds characters into lexer.
pub fn lex<I: IntoIterator<Item = char>, O: Observe<(Position, Token)>>(
    state: &mut State,
    position: &mut Position,
    token_position: &mut Position,
    buffer: &mut String,
    codepoint: &mut u32,
    source: I,
    dest: &mut O,
) -> Result<(), LexError> {
    use LexErrorKind as K;
    use State as S;
    use Token as T;
    let s = state;
    let p = position;
    let t = token_position;
    let b = buffer;
    let cp = codepoint;
    let d = dest;
    for c in source {
        p.advance(c == '\n');
        match *s {
            S::Initial => match c {
                ' ' | '\r' | '\n' | '\t' => (),
                '/' => *s = S::PrepareComment,
                '[' => d.update((*p, T::OpenSet)),
                ']' => d.update((*p, T::CloseSet)),
                '=' => *s = S::PrepareAssignment,
                '\'' => {
                    *t = *p;
                    *s = S::SingleQuote;
                }
                '"' => {
                    *t = *p;
                    *s = S::DoubleQuote;
                }
                '0'..='9' => {
                    *t = *p;
                    b.clear();
                    b.push(c);
                    *s = S::Integer;
                }
                'A'..='Z' | 'a'..='z' | '_' => {
                    *t = *p;
                    b.clear();
                    b.push(c);
                    *s = S::Identifier;
                }
                ',' => d.update((*p, T::Separator)),
                '<' => *s = S::PrepareOpenTag,
                '?' => *s = S::PrepareCloseTag,
                _ => return Err(LexError::new(*p, *s, K::Unexpected(c))),
            },
            S::PrepareComment => match c {
                '/' => *s = S::LineComment,
                '*' => *s = S::MultiLineComment,
                _ => return Err(LexError::new(*p, *s, K::Unexpected(c))),
            },
            S::PrepareAssignment => match c {
                '>' => {
                    d.update((*p, T::Assignment));
                    *s = S::Initial;
                }
                _ => return Err(LexError::new(*p, *s, K::Unexpected(c))),
            },
            S::SingleQuote => match c {
                '\'' => {
                    d.update((*t, T::SingleQuote(b.clone())));
                    *s = S::Initial;
                }
                '\\' => *s = S::SingleQuoteEscape,
                _ => b.push(c),
            },
            S::DoubleQuote => match c {
                '"' => {
                    d.update((*t, T::DoubleQuote(b.clone())));
                    *s = S::Initial;
                }
                '\\' => *s = S::DoubleQuoteEscape,
                _ => b.push(c),
            },
            S::Integer => match c {
                ' ' | '\r' | '\n' | '\t' | ';' => {
                    d.update((*t, T::Int(b.clone())));
                    *s = S::Initial;
                }
                '/' => {
                    d.update((*t, T::Int(b.clone())));
                    *s = S::PrepareComment;
                }
                '[' => {
                    d.update((*t, T::Int(b.clone())));
                    d.update((*p, T::OpenSet));
                    *s = S::Initial;
                }
                ']' => {
                    d.update((*t, T::Int(b.clone())));
                    d.update((*p, T::CloseSet));
                    *s = S::Initial;
                }
                '=' => {
                    d.update((*t, T::Int(b.clone())));
                    *s = S::PrepareAssignment;
                }
                '\'' => {
                    d.update((*t, T::Int(b.clone())));
                    b.clear();
                    *t = *p;
                    *s = S::SingleQuote;
                }
                ',' => {
                    d.update((*t, T::Int(b.clone())));
                    d.update((*p, T::Separator));
                    *s = S::Initial;
                }
                '"' => {
                    d.update((*t, T::Int(b.clone())));
                    b.clear();
                    *t = *p;
                    *s = S::DoubleQuote;
                }
                '0'..='0' | 'E' | 'e' | '+' => b.push(c),
                '-' | '.' => {
                    b.push(c);
                    *s = S::Decimal;
                }
                _ => return Err(LexError::new(*p, *s, K::Unexpected(c))),
            },
            S::IntegerDecimal => match c {
                '0'..='9' => {
                    b.push(c);
                    *s = S::Integer;
                }
                '.' => {
                    b.push('.');
                    *s = S::Decimal;
                }
                _ => return Err(LexError::new(*p, *s, K::Unexpected(c))),
            },
            S::Identifier => match c {
                ' ' | '\r' | '\n' | '\t' | ';' => {
                    d.update((*t, T::Identifier(b.clone())));
                    *s = S::Initial;
                }
                '/' => {
                    d.update((*t, T::Identifier(b.clone())));
                    *s = S::PrepareComment;
                }
                '[' => {
                    d.update((*t, T::Identifier(b.clone())));
                    d.update((*p, T::OpenSet));
                    *s = S::Initial;
                }
                ']' => {
                    d.update((*t, T::Identifier(b.clone())));
                    d.update((*p, T::CloseSet));
                    *s = S::Initial;
                }
                '=' => {
                    d.update((*t, T::Identifier(b.clone())));
                    *s = S::PrepareAssignment;
                }
                '\'' => {
                    d.update((*t, T::Identifier(b.clone())));
                    b.clear();
                    *t = *p;
                    *s = S::SingleQuote;
                }
                '"' => {
                    d.update((*t, T::Identifier(b.clone())));
                    b.clear();
                    *t = *p;
                    *s = S::DoubleQuote;
                }
                ',' => {
                    d.update((*t, T::Identifier(b.clone())));
                    d.update((*p, T::Separator));
                    *s = S::Initial;
                }
                '<' => {
                    d.update((*t, T::Identifier(b.clone())));
                    *s = S::PrepareOpenTag;
                }
                '?' => {
                    d.update((*t, T::Identifier(b.clone())));
                    *s = S::PrepareCloseTag;
                }
                '0'..='9' | '_' | 'a'..='z' | 'A'..='Z' => b.push(c),
                _ => return Err(LexError::new(*p, *s, K::Unexpected(c))),
            },
            S::PrepareOpenTag => match c {
                '?' => *s = S::PHPTag0,
                _ => return Err(LexError::new(*p, *s, K::Unexpected(c))),
            },
            S::PrepareCloseTag => match c {
                '>' => *s = S::Initial,
                _ => return Err(LexError::new(*p, *s, K::Unexpected(c))),
            },
            S::LineComment => {
                if c == '\n' {
                    *s = S::Initial
                }
            }
            S::MultiLineComment => {
                if c == '*' {
                    *s = S::MultiLineCommentPrepareExit
                }
            }
            S::MultiLineCommentPrepareExit => match c {
                '*' => (),
                '/' => *s = S::Initial,
                _ => *s = S::MultiLineComment,
            },
            S::Decimal => match c {
                ' ' | '\r' | '\n' | '\t' | ';' => {
                    d.update((*t, T::Float(b.clone())));
                    *s = S::Initial;
                }
                '/' => {
                    d.update((*t, T::Float(b.clone())));
                    *s = S::PrepareComment;
                }
                '[' => {
                    d.update((*t, T::Float(b.clone())));
                    d.update((*p, T::OpenSet));
                    *s = S::Initial;
                }
                ']' => {
                    d.update((*t, T::Float(b.clone())));
                    d.update((*p, T::CloseSet));
                    *s = S::Initial;
                }
                '=' => {
                    d.update((*t, T::Float(b.clone())));
                    *s = S::PrepareAssignment;
                }
                '\'' => {
                    d.update((*t, T::Float(b.clone())));
                    b.clear();
                    *t = *p;
                    *s = S::SingleQuote;
                }
                ',' => {
                    d.update((*t, T::Float(b.clone())));
                    d.update((*p, T::Separator));
                    *s = S::Initial;
                }
                '"' => {
                    d.update((*t, T::Float(b.clone())));
                    b.clear();
                    *t = *p;
                    *s = S::DoubleQuote;
                }
                '0'..='0' | 'E' | 'e' | '+' | '-' | '.' => b.push(c),
                _ => return Err(LexError::new(*p, *s, K::Unexpected(c))),
            },
            S::SingleQuoteEscape => match c {
                '\\' | '\'' => {
                    b.push(c);
                    *s = S::SingleQuote;
                }
                _ => {
                    b.push('\\');
                    b.push(c);
                    *s = S::SingleQuote;
                }
            },
            S::DoubleQuoteEscape => match c {
                'a' => {
                    b.push('\u{0007}');
                    *s = S::DoubleQuote;
                }
                'c' => *s = S::DoubleQuoteEscapeControl,
                'e' => {
                    b.push('\u{001b}');
                    *s = S::DoubleQuote;
                }
                'f' => {
                    b.push('\u{000c}');
                    *s = S::DoubleQuote;
                }
                'n' => {
                    b.push('\n');
                    *s = S::DoubleQuote;
                }
                'r' => {
                    b.push('\r');
                    *s = S::DoubleQuote;
                }
                't' => {
                    b.push('\t');
                    *s = S::DoubleQuote;
                }
                '$' => {
                    b.push('$');
                    *s = S::DoubleQuote;
                }
                '"' => {
                    b.push('"');
                    *s = S::DoubleQuote;
                }
                '\\' => {
                    b.push('\\');
                    *s = S::DoubleQuote;
                }
                'x' => {
                    *cp = 0;
                    *s = S::DoubleQuoteEscapeHex;
                }
                '0'..='9' => {
                    *cp = (c as u32) - ('0' as u32);
                    *s = S::DoubleQuoteEscapeOctal2;
                }
                _ => return Err(LexError::new(*p, *s, K::Unexpected(c))),
            },
            S::DoubleQuoteEscapeControl => {
                match char::try_from((c.to_uppercase().next().unwrap() as u32) ^ 0x60) {
                    Ok(x) => {
                        b.push(x);
                        *s = S::DoubleQuote;
                    }
                    Err(_) => {
                        return Err(LexError::new(
                            *p,
                            *s,
                            K::InvalidUnicode((c.to_uppercase().next().unwrap() as u32) ^ 0x60),
                        ))
                    }
                }
            }
            S::DoubleQuoteEscapeHex => match c {
                '{' => *s = S::DoubleQuoteEscapeHexBound,
                '0'..='9' => {
                    *cp = *cp | ((c as u32) - '0' as u32);
                    *s = S::DoubleQuoteEscapeHex2;
                }
                'a'..='f' => {
                    *cp = *cp | ((c as u32) - 'a' as u32);
                    *s = S::DoubleQuoteEscapeHex2;
                }
                'A'..='F' => {
                    *cp = *cp | ((c as u32) - 'A' as u32);
                    *s = S::DoubleQuoteEscapeHex2;
                }
                _ => return Err(LexError::new(*p, *s, K::Unexpected(c))),
            },
            S::DoubleQuoteEscapeOctal2 => match c {
                '0'..='7' => {
                    *cp = (*cp << 3) | ((c as u32) - ('0' as u32));
                    *s = S::DoubleQuoteEscapeOctal3;
                }
                '\\' => {
                    b.push(u8::try_from(*cp).unwrap().into());
                    *s = S::DoubleQuoteEscape;
                }
                '"' => {
                    b.push(u8::try_from(*cp).unwrap().into());
                    d.update((*t, T::DoubleQuote(b.clone())));
                    *s = S::Initial;
                }
                _ => {
                    b.push(u8::try_from(*cp).unwrap().into());
                    b.push(c);
                    *s = S::DoubleQuote;
                }
            },
            S::DoubleQuoteEscapeHexBound => match c {
                '0'..='9' => {
                    *cp = (*cp << 4) | ((c as u32) - '0' as u32);
                    *s = S::DoubleQuoteEscapeHex2;
                }
                'a'..='f' => {
                    *cp = (*cp << 4) | ((c as u32) - 'a' as u32);
                    *s = S::DoubleQuoteEscapeHex2;
                }
                'A'..='F' => {
                    *cp = (*cp << 4) | ((c as u32) - 'A' as u32);
                    *s = S::DoubleQuoteEscapeHex2;
                }
                '}' => match char::try_from(*cp) {
                    Ok(x) => b.push(x),
                    Err(_) => return Err(LexError::new(*p, *s, K::InvalidUnicode(*cp))),
                },
                _ => return Err(LexError::new(*p, *s, K::Unexpected(c))),
            },
            S::DoubleQuoteEscapeHex2 => match c {
                '0'..='9' => {
                    *cp = (*cp << 4) | ((c as u32) - '0' as u32);
                    b.push(u8::try_from(*cp).unwrap().into());
                    *s = S::DoubleQuote;
                }
                'a'..='f' => {
                    *cp = (*cp << 4) | ((c as u32) - 'a' as u32);
                    b.push(u8::try_from(*cp).unwrap().into());
                    *s = S::DoubleQuote;
                }
                'A'..='F' => {
                    *cp = (*cp << 4) | ((c as u32) - 'A' as u32);
                    b.push(u8::try_from(*cp).unwrap().into());
                    *s = S::DoubleQuote;
                }
                '\\' => {
                    b.push(u8::try_from(*cp).unwrap().into());
                    *s = S::DoubleQuoteEscape;
                }
                '"' => {
                    b.push(u8::try_from(*cp).unwrap().into());
                    d.update((*t, T::DoubleQuote(b.clone())));
                    *s = S::Initial;
                }
                _ => {
                    b.push(u8::try_from(*cp).unwrap().into());
                    b.push(c);
                    *s = S::DoubleQuote;
                }
            },
            S::PHPTag0 => match c {
                '=' => *s = S::Initial,
                'p' => *s = S::PHPTag1,
                _ => return Err(LexError::new(*p, *s, K::Unexpected(c))),
            },
            S::PHPTag1 => match c {
                'h' => *s = S::PHPTag2,
                _ => return Err(LexError::new(*p, *s, K::Unexpected(c))),
            },
            S::PHPTag2 => match c {
                'p' => *s = S::Initial,
                _ => return Err(LexError::new(*p, *s, K::Unexpected(c))),
            },
            S::DoubleQuoteEscapeOctal3 => match c {
                '0'..='7' => {
                    *cp = (*cp << 3) | ((c as u32) - ('0' as u32));
                    b.push(u8::try_from(*cp).unwrap().into());
                    *s = S::DoubleQuote;
                }
                '\\' => {
                    b.push(u8::try_from(*cp).unwrap().into());
                    *s = S::DoubleQuoteEscape;
                }
                '"' => {
                    b.push(u8::try_from(*cp).unwrap().into());
                    d.update((*t, T::DoubleQuote(b.clone())));
                    *s = S::Initial;
                }
                _ => {
                    b.push(u8::try_from(*cp).unwrap().into());
                    b.push(c);
                    *s = S::DoubleQuote;
                }
            },
        }
    }
    Ok(())
}

/// Feeds end-of-file into lexer.
pub fn eof<O: Observe<(Position, Token)>>(
    state: State,
    position: Position,
    token_position: Position,
    buffer: String,
    dest: &mut O,
) -> Result<(), LexError> {
    use LexError as E;
    use LexErrorKind as K;
    use State as S;
    use Token as T;
    let s = state;
    let p = position;
    let tp = token_position;
    let b = buffer;
    let d = dest;
    match s {
        S::Initial | S::LineComment | S::MultiLineComment | S::MultiLineCommentPrepareExit => (),
        S::PrepareComment
        | S::PrepareAssignment
        | S::SingleQuote
        | S::DoubleQuote
        | S::IntegerDecimal
        | S::PrepareOpenTag
        | S::PrepareCloseTag
        | S::SingleQuoteEscape
        | S::DoubleQuoteEscape
        | S::DoubleQuoteEscapeControl
        | S::DoubleQuoteEscapeHex
        | S::DoubleQuoteEscapeOctal2
        | S::DoubleQuoteEscapeHexBound
        | S::DoubleQuoteEscapeHex2
        | S::PHPTag0
        | S::PHPTag1
        | S::PHPTag2
        | S::DoubleQuoteEscapeOctal3 => return Err(E::new(p, s, K::EOF)),
        S::Integer => d.update((tp, T::Int(b))),
        S::Identifier => d.update((tp, T::Identifier(b))),
        S::Decimal => d.update((tp, T::Float(b))),
    }
    Ok(())
}
