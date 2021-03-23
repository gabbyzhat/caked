use crate::{Floating, Integral};

/// A value of a node.
#[derive(Debug)]
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
    Set(Vec<Node>),
}

/// A graph node.
#[derive(Debug)]
pub struct Node {
    /// The name of this node, if any.
    pub name: Option<String>,

    /// The value of this node.
    pub value: Value,
}
