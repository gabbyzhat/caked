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
    pub fn new(key: Option<String>, value: Value) -> Self {
        Self { key, value }
    }
}
