//! CakePHP style configuration reading and writing
//!
//! # Examples
//!
//! ```
//! // tbi
//! ```

#![forbid(unsafe_code, future_incompatible, rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
//#![warn(missing_docs, missing_doc_code_examples, unreachable_pub)]

pub(crate) mod lex;

/// Key-value pair.
pub(crate) mod kvp;

pub use kvp::{Value, KeyValuePair};
pub fn read() {

}