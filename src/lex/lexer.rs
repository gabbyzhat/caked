use super::{Flags, Position, State, Token};
use std::result::Result;


/// A lexer.
#[derive(Debug)]
pub struct Lexer<R> {
    flags: Flags,
    state: State,
    position: Position,
    token_position: Position,
    buffer: String,
    codepoint: u32,
    reader: R,
}

