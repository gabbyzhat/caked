use std::collections::VecDeque;
use std::convert::TryFrom;
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Copy)]
pub struct Position {
    pub index: usize,
    pub line: usize,
    pub column: usize,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Line {}, Column {}", self.line, self.column)
    }
}

impl Position {
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

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum FeedErrorKind {
    EOF,
    Unexpected(char),
    InvalidUnicode(u32),
}

/// A feed error.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct FeedError {
    pub position: Position,
    pub state: State,
    pub got: FeedErrorKind,
}

impl FeedError {
    pub fn new(position: Position, state: State, got: FeedErrorKind) -> Self {
        Self {
            position,
            state,
            got,
        }
    }
}

impl fmt::Display for FeedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let Self {
            position,
            state,
            got,
        } = self;
        use FeedErrorKind as K;
        match got {
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

/// Feeds source into lexer.
pub fn feed<I: IntoIterator<Item = char>>(
    state: &mut State,
    position: &mut Position,
    token_position: &mut Position,
    buffer: &mut String,
    codepoint: &mut u32,
    source: I,
    dest: &mut VecDeque<(Position, Token)>,
) -> Result<(), FeedError> {
    use FeedErrorKind as K;
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
                '[' => d.push_back((*p, T::OpenSet)),
                ']' => d.push_back((*p, T::CloseSet)),
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
                ',' => d.push_back((*p, T::Separator)),
                '<' => *s = S::PrepareOpenTag,
                '?' => *s = S::PrepareCloseTag,
                _ => return Err(FeedError::new(*p, *s, K::Unexpected(c))),
            },
            S::PrepareComment => match c {
                '/' => *s = S::LineComment,
                '*' => *s = S::MultiLineComment,
                _ => return Err(FeedError::new(*p, *s, K::Unexpected(c))),
            },
            S::PrepareAssignment => match c {
                '>' => {
                    d.push_back((*p, T::Assignment));
                    *s = S::Initial;
                }
                _ => return Err(FeedError::new(*p, *s, K::Unexpected(c))),
            },
            S::SingleQuote => match c {
                '\'' => {
                    d.push_back((*t, T::SingleQuote(b.clone())));
                    *s = S::Initial;
                }
                '\\' => *s = S::SingleQuoteEscape,
                _ => b.push(c),
            },
            S::DoubleQuote => match c {
                '"' => {
                    d.push_back((*t, T::DoubleQuote(b.clone())));
                    *s = S::Initial;
                }
                '\\' => *s = S::DoubleQuoteEscape,
                _ => b.push(c),
            },
            S::Integer => match c {
                ' ' | '\r' | '\n' | '\t' | ';' => {
                    d.push_back((*t, T::Integral(b.clone())));
                    *s = S::Initial;
                }
                '/' => {
                    d.push_back((*t, T::Integral(b.clone())));
                    *s = S::PrepareComment;
                }
                '[' => {
                    d.push_back((*t, T::Integral(b.clone())));
                    d.push_back((*p, T::OpenSet));
                    *s = S::Initial;
                }
                ']' => {
                    d.push_back((*t, T::Integral(b.clone())));
                    d.push_back((*p, T::CloseSet));
                    *s = S::Initial;
                }
                '=' => {
                    d.push_back((*t, T::Integral(b.clone())));
                    *s = S::PrepareAssignment;
                }
                '\'' => {
                    d.push_back((*t, T::Integral(b.clone())));
                    b.clear();
                    *t = *p;
                    *s = S::SingleQuote;
                }
                ',' => {
                    d.push_back((*t, T::Integral(b.clone())));
                    d.push_back((*p, T::Separator));
                    *s = S::Initial;
                }
                '"' => {
                    d.push_back((*t, T::Integral(b.clone())));
                    b.clear();
                    *t = *p;
                    *s = S::DoubleQuote;
                }
                '0'..='0' | 'E' | 'e' | '+' => b.push(c),
                '-' | '.' => {
                    b.push(c);
                    *s = S::Decimal;
                }
                _ => return Err(FeedError::new(*p, *s, K::Unexpected(c))),
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
                _ => return Err(FeedError::new(*p, *s, K::Unexpected(c))),
            },
            S::Identifier => match c {
                ' ' | '\r' | '\n' | '\t' | ';' => {
                    d.push_back((*t, T::Identifier(b.clone())));
                    *s = S::Initial;
                }
                '/' => {
                    d.push_back((*t, T::Identifier(b.clone())));
                    *s = S::PrepareComment;
                }
                '[' => {
                    d.push_back((*t, T::Identifier(b.clone())));
                    d.push_back((*p, T::OpenSet));
                    *s = S::Initial;
                }
                ']' => {
                    d.push_back((*t, T::Identifier(b.clone())));
                    d.push_back((*p, T::CloseSet));
                    *s = S::Initial;
                }
                '=' => {
                    d.push_back((*t, T::Identifier(b.clone())));
                    *s = S::PrepareAssignment;
                }
                '\'' => {
                    d.push_back((*t, T::Identifier(b.clone())));
                    b.clear();
                    *t = *p;
                    *s = S::SingleQuote;
                }
                '"' => {
                    d.push_back((*t, T::Identifier(b.clone())));
                    b.clear();
                    *t = *p;
                    *s = S::DoubleQuote;
                }
                ',' => {
                    d.push_back((*t, T::Identifier(b.clone())));
                    d.push_back((*p, T::Separator));
                    *s = S::Initial;
                }
                '<' => {
                    d.push_back((*t, T::Identifier(b.clone())));
                    *s = S::PrepareOpenTag;
                }
                '?' => {
                    d.push_back((*t, T::Identifier(b.clone())));
                    *s = S::PrepareCloseTag;
                }
                '0'..='9' | '_' | 'a'..='z' | 'A'..='Z' => b.push(c),
                _ => return Err(FeedError::new(*p, *s, K::Unexpected(c))),
            },
            S::PrepareOpenTag => match c {
                '?' => *s = S::PHPTag0,
                _ => return Err(FeedError::new(*p, *s, K::Unexpected(c))),
            },
            S::PrepareCloseTag => match c {
                '>' => *s = S::Initial,
                _ => return Err(FeedError::new(*p, *s, K::Unexpected(c))),
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
                    d.push_back((*t, T::Float(b.clone())));
                    *s = S::Initial;
                }
                '/' => {
                    d.push_back((*t, T::Float(b.clone())));
                    *s = S::PrepareComment;
                }
                '[' => {
                    d.push_back((*t, T::Float(b.clone())));
                    d.push_back((*p, T::OpenSet));
                    *s = S::Initial;
                }
                ']' => {
                    d.push_back((*t, T::Float(b.clone())));
                    d.push_back((*p, T::CloseSet));
                    *s = S::Initial;
                }
                '=' => {
                    d.push_back((*t, T::Float(b.clone())));
                    *s = S::PrepareAssignment;
                }
                '\'' => {
                    d.push_back((*t, T::Float(b.clone())));
                    b.clear();
                    *t = *p;
                    *s = S::SingleQuote;
                }
                ',' => {
                    d.push_back((*t, T::Float(b.clone())));
                    d.push_back((*p, T::Separator));
                    *s = S::Initial;
                }
                '"' => {
                    d.push_back((*t, T::Float(b.clone())));
                    b.clear();
                    *t = *p;
                    *s = S::DoubleQuote;
                }
                '0'..='0' | 'E' | 'e' | '+' | '-' | '.' => b.push(c),
                _ => return Err(FeedError::new(*p, *s, K::Unexpected(c))),
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
                _ => return Err(FeedError::new(*p, *s, K::Unexpected(c))),
            },
            S::DoubleQuoteEscapeControl => {
                match char::try_from((c.to_uppercase().next().unwrap() as u32) ^ 0x60) {
                    Ok(x) => {
                        b.push(x);
                        *s = S::DoubleQuote;
                    }
                    Err(_) => {
                        return Err(FeedError::new(
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
                _ => return Err(FeedError::new(*p, *s, K::Unexpected(c))),
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
                    d.push_back((*t, T::DoubleQuote(b.clone())));
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
                    Err(_) => return Err(FeedError::new(*p, *s, K::InvalidUnicode(*cp))),
                },
                _ => return Err(FeedError::new(*p, *s, K::Unexpected(c))),
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
                    d.push_back((*t, T::DoubleQuote(b.clone())));
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
                _ => return Err(FeedError::new(*p, *s, K::Unexpected(c))),
            },
            S::PHPTag1 => match c {
                'h' => *s = S::PHPTag2,
                _ => return Err(FeedError::new(*p, *s, K::Unexpected(c))),
            },
            S::PHPTag2 => match c {
                'p' => *s = S::Initial,
                _ => return Err(FeedError::new(*p, *s, K::Unexpected(c))),
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
                    d.push_back((*t, T::DoubleQuote(b.clone())));
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

pub fn eof(
    state: State,
    position: Position,
    token_position: Position,
    buffer: String,
    dest: &mut VecDeque<(Position, Token)>,
) -> Result<(), FeedError> {
    use FeedError as E;
    use FeedErrorKind as K;
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
        S::Integer => d.push_back((tp, T::Integral(b))),
        S::Identifier => d.push_back((tp, T::Identifier(b))),
        S::Decimal => d.push_back((tp, T::Float(b))),
    }
    Ok(())
}
