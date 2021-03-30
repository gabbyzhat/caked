use super::{Position, State, Token};
use std::result::Result;

/// A lexer.
#[derive(Debug)]
pub struct Lexer<R> {
    state: State,
    position: Position,
    token_position: Position,
    buffer: String,
    codepoint: u32,
    reader: R,
}

