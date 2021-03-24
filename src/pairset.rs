use crate::{Floating, Integral};

/// A value of a node.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Value {
    /// Null.
    Null,

    /// Boolean.
    Bool(bool),

    /// Integral.
    Int(Integral),

    /// Floating point.
    Float(Floating),

    /// String.
    Str(String),

    /// Set of nodes.
    Set(Vec<KeyValuePair>),
}

/// A graph node.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyValuePair {
    /// The name of this node, if any.
    pub key: Option<String>,

    /// The value of this node.
    pub value: Value,
}

