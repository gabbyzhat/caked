//! CakePHP style configuration reading and writing
//!
//! # Examples
//!
//! ```
//! // tbi
//! ```

#![forbid(unsafe_code, future_incompatible, rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, missing_doc_code_examples, unreachable_pub)]

pub(crate) mod lex;

pub(crate) mod kvp;

pub(crate) mod deser;

pub(crate) mod ser;

pub use deser::{deser_file, deser_str, DeserError, DeserErrorKind};
pub use kvp::{KeyValuePair, Value};
pub use lex::{eof, lex, LexError, LexErrorKind, Position, Token};
pub use ser::{ser_file, ser_str, ser_write};
