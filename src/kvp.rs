use std::fmt::{Display, Error as FmtError, Formatter};

/// A value of a node.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    /// Null.
    Null,

    /// Boolean.
    Bool(bool),

    /// Integral.
    Int(i64),

    /// Floating point.
    Float(f64),

    /// String.
    Str(String),

    /// Set of nodes.
    Set(Vec<KeyValuePair>),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        use Value as V;
        match self {
            V::Null => write!(f, "null"),
            V::Bool(true) => write!(f, "true"),
            V::Bool(false) => write!(f, "false"),
            V::Int(x) => write!(f, "{}", x),
            V::Float(x) => {
                let s = format!("{}", x);
                if s.contains('.') {
                    write!(f, "{}", s)
                } else {
                    write!(f, "{}.0", s)
                }
            }
            V::Str(x) => write!(f, "{}", php_str(x)),
            V::Set(x) => {
                write!(f, "[")?;
                for kvp in x {
                    write!(f, "{},", kvp)?;
                }
                write!(f, "]")
            }
        }
    }
}

enum PHPStringState {
    Undecided,
    UndecidedEscape,
    Decided,
}

fn php_str(input: &str) -> String {
    use PHPStringState as P;
    let mut d = String::new();
    let mut s = String::new();
    let mut state = PHPStringState::Undecided;
    d.reserve(input.len() + 2);
    s.reserve(input.len() + 2);
    d.push('"');
    s.push('\'');
    for c in input.chars() {
        match state {
            P::Undecided => match c {
                '\0' => {
                    state = P::Decided;
                    d.push_str("\\000");
                }
                '\u{0007}' => {
                    state = P::Decided;
                    d.push_str("\\a");
                }
                '\u{001b}' => {
                    state = P::Decided;
                    d.push_str("\\e");
                }
                '\u{000c}' => {
                    state = P::Decided;
                    d.push_str("\\f");
                }
                '\r' => {
                    state = P::Decided;
                    d.push_str("\\r");
                }
                '\n' => {
                    state = P::Decided;
                    d.push_str("\\n");
                }
                '\t' => {
                    state = P::Decided;
                    d.push_str("\\t");
                }
                '\'' => {
                    s.push_str("\\'");
                    d.push('\'');
                }
                '\\' => {
                    state = P::UndecidedEscape;
                    d.push_str("\\\\");
                }
                '"' => {
                    s.push('"');
                    d.push_str("\\\"");
                }
                '$' => {
                    s.push('$');
                    d.push_str("\\$");
                }
                _ => {
                    s.push(c);
                    d.push(c);
                }
            },
            P::UndecidedEscape => match c {
                '\0' => {
                    state = P::Decided;
                    d.push_str("\\000");
                }
                '\u{0007}' => {
                    state = P::Decided;
                    d.push_str("\\a");
                }
                '\u{001b}' => {
                    state = P::Decided;
                    d.push_str("\\e");
                }
                '\u{000c}' => {
                    state = P::Decided;
                    d.push_str("\\f");
                }
                '\r' => {
                    state = P::Decided;
                    d.push_str("\\r");
                }
                '\n' => {
                    state = P::Decided;
                    d.push_str("\\n");
                }
                '\t' => {
                    state = P::Decided;
                    d.push_str("\\t");
                }
                '\'' => {
                    state = P::Undecided;
                    s.push_str("\\\\\\'");
                    d.push('\'');
                }
                '\\' => {
                    s.push_str("\\\\");
                    d.push_str("\\\\\\\\");
                }
                '"' => {
                    state = P::Undecided;
                    s.push_str("\\\"");
                    d.push_str("\\\"");
                }
                '$' => {
                    state = P::Undecided;
                    s.push_str("\\$");
                    d.push_str("\\$");
                }
                _ => {
                    state = P::Undecided;
                    s.push('\\');
                    s.push(c);
                    d.push(c);
                }
            },
            P::Decided => match c {
                '\0' => d.push_str("\\000"),

                '\u{0007}' => d.push_str("\\a"),
                '\u{001b}' => d.push_str("\\e"),
                '\u{000c}' => d.push_str("\\f"),
                '\r' => d.push_str("\\r"),
                '\n' => d.push_str("\\n"),
                '\t' => d.push_str("\\t"),
                '\'' => d.push('\''),
                '\\' => d.push_str("\\\\"),
                '"' => d.push_str("\\\""),
                '$' => d.push_str("\\$"),
                _ => d.push(c),
            },
        }
    }

    match state {
        P::Undecided => {
            s.push('\'');
            s
        }
        P::UndecidedEscape => {
            s.push_str("\\\\'");
            s
        }
        P::Decided => {
            d.push('"');
            d
        }
    }
}

impl Eq for Value {}
impl Ord for Value {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

/// A graph node.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyValuePair {
    /// The name of this node, if any.
    pub key: Option<String>,

    /// The value of this node.
    pub value: Value,
}

impl KeyValuePair {
    /// Creates a new key-value pair.
    pub fn new(key: Option<String>, value: Value) -> Self {
        Self { key, value }
    }

    /// Applies the key prefix, if any
    pub fn key_prefix(&self) -> String {
        if let Some(key) = &self.key {
            let mut buf = String::new();
            buf.push_str(&php_str(key));
            buf.push_str(" => ");
            buf
        } else {
            String::new()
        }
    }
}

impl Display for KeyValuePair {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
        if let Some(key) = &self.key {
            write!(f, "{} => {}", php_str(key), self.value)
        } else {
            self.value.fmt(f)
        }
    }
}
