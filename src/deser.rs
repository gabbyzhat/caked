use crate::kvp::{KeyValuePair, Value};
use crate::lex::{eof, lex, LexError, LexErrorKind, Position, State, Token};
use std::collections::VecDeque;
use std::fs::File;
use std::io::Read;
use std::mem::swap;
use std::path::Path;
use std::str::FromStr;

/// Deserialization error kind.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum DeserErrorKind {
    /// Lexer Lex error.
    Lex(LexErrorKind),

    /// Unexpected identifier token.
    UnexpectedIdentifier(String),

    /// Invalid key token.
    InvalidKey,

    /// Missing comma.
    MissingComma,

    /// Float casting error.
    FloatCast(String),

    /// Integer casting error.
    IntCast(String),
}

/// Deserialization error.
#[derive(Debug, Clone)]
pub struct DeserError {
    /// Token position.
    pub position: Position,

    /// Error kind.
    pub kind: DeserErrorKind,
}

impl DeserError {
    /// Creates a new deserialization error.
    pub fn new(position: Position, kind: DeserErrorKind) -> Self {
        Self { position, kind }
    }
}

impl From<LexError> for DeserError {
    fn from(err: LexError) -> Self {
        Self::new(err.position, DeserErrorKind::Lex(err.kind))
    }
}

/// Deserialize from a file.
pub fn deser_file<P: AsRef<Path>>(path: P) -> Result<Vec<KeyValuePair>, DeserError> {
    // we could do this streaming but utf-8 reading is weird
    let mut f = File::open(path).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    deser_str(&s)
}

/// Deserialize from a string.
pub fn deser_str(input: &str) -> Result<Vec<KeyValuePair>, DeserError> {
    use DeserError as E;
    use DeserErrorKind as K;
    use Token as T;
    use Value as V;
    let mut tokens = VecDeque::new();
    {
        let mut s = State::default();
        let mut p = Position::default();
        let mut tp = Position::default();
        let mut b = String::new();
        let mut cp = 0;
        lex(
            &mut s,
            &mut p,
            &mut tp,
            &mut b,
            &mut cp,
            input.chars(),
            &mut tokens,
        )?;
        eof(s, p, tp, b, &mut tokens)?;
    };
    let mut data: Vec<(Option<String>, Vec<KeyValuePair>)> = Vec::new();
    let mut current: Vec<KeyValuePair> = Vec::new();
    let mut pair_key: Option<String> = None;
    let mut pair_value = V::Null;
    let mut pair_set = false;
    let mut lexer_break = false;
    let mut first_open = true;
    for (p, t) in tokens {
        match t {
            T::Float(x) => {
                if pair_set {
                    return Err(E::new(p, K::MissingComma));
                }
                match f64::from_str(&x) {
                    Ok(v) => pair_value = V::Float(v),
                    Err(_) => return Err(E::new(p, K::IntCast(x))),
                }
                pair_set = true;
            }
            T::Int(x) => {
                if pair_set {
                    return Err(E::new(p, K::MissingComma));
                }
                match i64::from_str(&x) {
                    Ok(v) => pair_value = V::Int(v),
                    Err(_) => return Err(E::new(p, K::IntCast(x))),
                }
                pair_set = true;
            }
            T::DoubleQuote(x) | T::SingleQuote(x) => {
                if pair_set {
                    return Err(E::new(p, K::MissingComma));
                }
                pair_value = V::Str(x);
                pair_set = true;
            }
            T::Assignment => {
                let mut pv = V::Null;
                swap(&mut pv, &mut pair_value);
                match pv {
                    V::Str(v) => pair_key = Some(v),
                    _ => return Err(E::new(p, K::InvalidKey)),
                }

                pair_value = V::Null;
                pair_set = false;
            }
            T::Identifier(x) => {
                let lower = x.to_lowercase();
                match lower.as_str() {
                    "true" => {
                        if pair_set {
                            return Err(E::new(p, K::MissingComma));
                        }
                        pair_value = V::Bool(true);
                        pair_set = true;
                    }
                    "false" => {
                        if pair_set {
                            return Err(E::new(p, K::MissingComma));
                        }
                        pair_value = V::Bool(false);
                        pair_set = true;
                    }
                    "null" => {
                        if pair_set {
                            return Err(E::new(p, K::MissingComma));
                        }
                        pair_value = V::Null;
                        pair_set = true;
                    }
                    "return" => (),
                    _ => return Err(E::new(p, K::UnexpectedIdentifier(x))),
                }
            }
            T::OpenSet => {
                if pair_set {
                    return Err(E::new(p, K::MissingComma));
                }
                if !first_open {
                    let mut old_current: Vec<KeyValuePair> = Vec::new();
                    swap(&mut old_current, &mut current);
                    let mut old_pair_key = None;
                    swap(&mut old_pair_key, &mut pair_key);
                    data.push((old_pair_key, old_current));
                    pair_value = V::Null;
                } else {
                    first_open = false;
                }
            }
            T::Separator => {
                println!("zeh");
                if pair_set {
                    let mut old_pair_key = None;
                    let mut old_pair_value = V::Null;
                    swap(&mut old_pair_key, &mut pair_key);
                    swap(&mut old_pair_value, &mut pair_value);
                    current.push(KeyValuePair::new(old_pair_key, old_pair_value));
                    pair_set = false;
                }
            }
            T::CloseSet => {
                if pair_set {
                    let mut old_pair_key = None;
                    let mut old_pair_value = V::Null;
                    swap(&mut old_pair_key, &mut pair_key);
                    swap(&mut old_pair_value, &mut pair_value);
                    current.push(KeyValuePair::new(old_pair_key, old_pair_value));
                    pair_set = false;
                }

                if data.len() == 0 {
                    lexer_break = true;
                } else {
                    if pair_set {
                        let mut old_pair_key = None;
                        let mut old_pair_value = V::Null;
                        swap(&mut old_pair_key, &mut pair_key);
                        swap(&mut old_pair_value, &mut pair_value);
                        current.push(KeyValuePair::new(old_pair_key, old_pair_value));
                        pair_set = false;
                    }
                    let (old_key, mut old_current) = data.pop().unwrap();
                    swap(&mut old_current, &mut current);
                    current.push(KeyValuePair::new(old_key, V::Set(old_current)));
                }
            }
        }
        if lexer_break {
            break;
        }
    }
    Ok(current)
}
