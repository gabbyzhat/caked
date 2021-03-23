use super::{Flags, Position, State, Token};
use std::result::Result;

/// A feed error.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct FeedError {
    pub position: Position,
    pub state: State,
    pub got: Option<char>,
}

impl FeedError {
    pub fn new(position: Position, state: State, got: Option<char>) -> Self {
        Self {
            position,
            state,
            got,
        }
    }
}

/// Feeds source into lexer.
pub(crate) fn feed<I: IntoIterator<Item = char>>(
    flags: &mut Flags,
    state: &mut State,
    position: &mut Position,
    token_position: &mut Position,
    buffer: &mut String,
    codepoint: &mut u32,
    source: I,
    dest: &mut Vec<(Position, Token)>,
) -> Result<(), FeedError> {
    use State as S;
    use Token as T;
    let f = flags;
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
                '[' => d.push((*p, T::OpenSet)),
                ']' => d.push((*p, T::CloseSet)),
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
                    b.clear();
                    b.push(c);
                    *s = S::Integer;
                }
                'A'..='Z' | 'a'..='z' | '_' => {
                    b.clear();
                    b.push(c);
                    *s = S::Identifier;
                }
                ',' => d.push((*p, T::Separator)),
                '<' => *s = S::PrepareOpenTag,
                '?' => *s = S::PrepareCloseTag,
                _ => return Err(FeedError::new(*p, *s, Some(c))),
            },
            S::PrepareComment => match c {
                '/' => *s = S::LineComment,
                '*' => *s = S::MultiLineComment,
                _ => return Err(FeedError::new(*p, *s, Some(c))),
            },
            S::PrepareAssignment => match c {
                '>' => {
                    d.push((*p, T::Assignment));
                    *s = S::Initial;
                }
                _ => return Err(FeedError::new(*p, *s, Some(c))),
            },
            S::SingleQuote => match c {
                '\'' => {
                    d.push((*t, T::SingleQuote(b.clone())));
                    *s = S::Initial;
                }
                '\\' => *s = S::SingleQuoteEscape,
                _ => b.push(c),
            },
            _ => (),
        }
    }
    Ok(())
}

pub fn eof(
    flags: &mut Flags,
    state: &mut State,
    position: &mut Position,
    token_position: &mut Position,
    buffer: &mut String,
    codepoint: &mut u32,
    dest: &mut Vec<(Position, Token)>,
) -> Result<(), FeedError> {
    Ok(())
}
